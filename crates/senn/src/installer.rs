//! First install of a host with nixos-anywhere.
//!
//! Runs outside of OpenTofu, so no secret ever enters the tofu state. The install key and
//! the host secrets exist only in a private temporary directory during the install.

use std::fs::{OpenOptions, Permissions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::error::{Error, Result};
use crate::manifest::Manifest;
use crate::runner::{Cmd, Runner};
use crate::secrets::Secrets;

/// Installs hosts of one herd.
pub struct Installer<'a> {
    runner: &'a dyn Runner,

    manifest: &'a Manifest,

    secrets: &'a Secrets<'a>,

    /// The herd flake, the source of each host's NixOS configuration.
    herd: &'a Path,
}

impl<'a> Installer<'a> {
    /// An installer for the hosts of the herd flake at `herd`.
    #[must_use]
    pub const fn new(runner: &'a dyn Runner, manifest: &'a Manifest, secrets: &'a Secrets<'a>, herd: &'a Path) -> Self {
        Self {
            runner,
            manifest,
            secrets,
            herd,
        }
    }

    /// Checks whether `host` is installed: root SSH to it over the tailnet works.
    ///
    /// A fresh machine is not on the tailnet yet. An installed host doesn't accept the
    /// install key. So a wrong "not installed" can't lead to wiping a running host.
    ///
    /// # Errors
    ///
    /// The errors of [`Runner::exec`] if `ssh` can't start. A failed connection is `false`.
    pub fn is_installed(&self, host: &str) -> Result<bool> {
        let cmd = Cmd::new(&self.manifest.tools.ssh).args([
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=5",
            "-o",
            "StrictHostKeyChecking=accept-new",
            &format!("root@{host}"),
            "true",
        ]);

        Ok(self.runner.exec(&cmd)?.code == Some(0))
    }

    /// Waits until `host` answers over the tailnet, probing every `interval` for at most
    /// `timeout`. A fresh install needs this: the host reboots and joins the tailnet on its own.
    ///
    /// # Errors
    ///
    /// The errors of [`Installer::is_installed`], and [`Error::NotOnTailnet`] after `timeout`.
    pub fn wait_until_installed(&self, host: &str, timeout: Duration, interval: Duration) -> Result<()> {
        let start = Instant::now();
        loop {
            if self.is_installed(host)? {
                return Ok(());
            }
            if start.elapsed() >= timeout {
                return Err(Error::NotOnTailnet(host.to_owned()));
            }
            std::thread::sleep(interval);
        }
    }

    /// Installs NixOS on `host` at `ipv4` with nixos-anywhere.
    ///
    /// The install key and the host secrets are written to a temporary directory with mode
    /// 0700, and each file with mode 0600. The secrets go to `/var/lib/senn/secrets` on the
    /// host. The directory is removed when this method returns, also on errors.
    ///
    /// # Errors
    ///
    /// The errors of [`Secrets::read`], [`crate::Error::UnknownHost`], [`crate::Error::Io`]
    /// for the temporary files, and the errors of [`Runner::run`] if nixos-anywhere fails.
    pub fn install(&self, host: &str, ipv4: &str) -> Result<()> {
        let tmp = tempfile::tempdir()?;

        let key = tmp.path().join("install-key");
        let mut private = self.secrets.read(&self.manifest.install_key)?;
        if !private.ends_with('\n') {
            // OpenSSH rejects a private key without a final newline.
            private.push('\n');
        }
        Self::write_private(&key, &private)?;

        let extra = tmp.path().join("extra");
        let dir = extra.join("var/lib/senn/secrets");
        Self::create_private_dir(&dir)?;
        for (name, reference) in &self.manifest.host(host)?.secrets {
            Self::write_private(&dir.join(name.as_str()), &self.secrets.read(reference)?)?;
        }

        self.runner
            .run(&Cmd::new(&self.manifest.tools.nixos_anywhere).args([
                "--flake".to_owned(),
                format!("{}#{host}", self.herd.display()),
                "--target-host".to_owned(),
                format!("root@{ipv4}"),
                "-i".to_owned(),
                key.display().to_string(),
                "--extra-files".to_owned(),
                extra.display().to_string(),
                "--build-on".to_owned(),
                "remote".to_owned(),
            ]))
            .map(drop)
    }

    /// Creates a new file only the owner can read (mode 0600).
    fn write_private(path: &Path, contents: &str) -> std::io::Result<()> {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)?
            .write_all(contents.as_bytes())
    }

    /// Creates `dir` and its parents; `dir` itself gets mode 0700.
    fn create_private_dir(dir: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        std::fs::set_permissions(dir, Permissions::from_mode(0o700))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::manifest::tests::SAMPLE;
    use crate::runner::fake::Fake;

    /// The permission bits of `path`.
    fn mode(path: &Path) -> u32 {
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn installed_means_reachable_over_the_tailnet() {
        let m = Manifest::parse(SAMPLE).unwrap();

        let up = Fake::new(|_| Some(String::new()));
        let secrets = Secrets::new(&up, "backend");
        assert!(
            Installer::new(&up, &m, &secrets, Path::new("/herd"))
                .is_installed("cloud-a")
                .unwrap()
        );
        assert_eq!(up.calls()[0].args.last().unwrap(), "true");
        assert!(up.calls()[0].args.contains(&"root@cloud-a".to_owned()));

        let down = Fake::new(|_| None);
        let secrets = Secrets::new(&down, "backend");
        assert!(
            !Installer::new(&down, &m, &secrets, Path::new("/herd"))
                .is_installed("cloud-a")
                .unwrap()
        );
    }

    #[test]
    fn waits_until_the_host_answers_over_the_tailnet() {
        let m = Manifest::parse(SAMPLE).unwrap();
        let probes = std::rc::Rc::new(std::cell::Cell::new(0));
        let counted = probes.clone();
        // The third probe succeeds.
        let fake = Fake::new(move |_| {
            counted.set(counted.get() + 1);
            (counted.get() >= 3).then(String::new)
        });
        let secrets = Secrets::new(&fake, "backend");
        let installer = Installer::new(&fake, &m, &secrets, Path::new("/herd"));

        installer
            .wait_until_installed("cloud-a", Duration::from_secs(60), Duration::ZERO)
            .unwrap();

        assert_eq!(probes.get(), 3);
    }

    #[test]
    fn gives_up_when_the_host_never_answers() {
        let m = Manifest::parse(SAMPLE).unwrap();
        let fake = Fake::new(|_| None);
        let secrets = Secrets::new(&fake, "backend");
        let installer = Installer::new(&fake, &m, &secrets, Path::new("/herd"));

        let result = installer.wait_until_installed("cloud-a", Duration::ZERO, Duration::ZERO);

        assert!(
            matches!(result, Err(Error::NotOnTailnet(ref h)) if h == "cloud-a"),
            "{result:?}"
        );
    }

    #[test]
    fn install_runs_nixos_anywhere_with_key_and_host_secrets() {
        let m = Manifest::parse(SAMPLE).unwrap();
        let seen = std::rc::Rc::new(std::cell::RefCell::new(None::<PathBuf>));
        let seen_in_fake = seen.clone();
        // Check the files while nixos-anywhere "runs": they are deleted afterwards.
        let fake = Fake::new(move |cmd| match cmd.program.as_str() {
            "backend" => {
                let req: serde_json::Value = serde_json::from_str(cmd.stdin.as_deref().unwrap()).unwrap();
                let value = format!("value-of-{}", req["ref"].as_str().unwrap());
                Some(serde_json::json!({ "value": value }).to_string())
            },
            "nixos-anywhere" => {
                let arg = |flag: &str| {
                    let i = cmd.args.iter().position(|a| a == flag).unwrap();
                    PathBuf::from(&cmd.args[i + 1])
                };
                let key = arg("-i");
                assert_eq!(std::fs::read_to_string(&key).unwrap(), "value-of-op://v/install/key\n");
                let dir = arg("--extra-files").join("var/lib/senn/secrets");
                let secret = dir.join("tailscale-authkey");
                assert_eq!(std::fs::read_to_string(&secret).unwrap(), "value-of-op://v/ts/key");
                assert_eq!(mode(&key), 0o600);
                assert_eq!(mode(&secret), 0o600);
                assert_eq!(mode(&dir), 0o700);
                *seen_in_fake.borrow_mut() = Some(key);
                Some(String::new())
            },
            _ => None,
        });
        let secrets = Secrets::new(&fake, "backend");

        Installer::new(&fake, &m, &secrets, Path::new("/herd"))
            .install("cloud-a", "1.2.3.4")
            .unwrap();

        let call = fake
            .calls()
            .into_iter()
            .find(|c| c.program == "nixos-anywhere")
            .unwrap();
        assert_eq!(
            &call.args[..4],
            ["--flake", "/herd#cloud-a", "--target-host", "root@1.2.3.4"]
        );
        assert!(call.args.windows(2).any(|w| w == ["--build-on", "remote"]));
        let key = seen.borrow().clone().unwrap();
        assert!(!key.exists(), "install key still on disk");
    }
}
