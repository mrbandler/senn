//! Configure stage: push host secrets over the tailnet, then deploy with deploy-rs.

use std::path::Path;

use crate::error::Result;
use crate::manifest::Manifest;
use crate::runner::{Cmd, Runner};
use crate::secrets::Secrets;

/// Where host secrets live on every host. The base aspect creates it with mode 0700.
const SECRETS_DIR: &str = "/var/lib/senn/secrets";

/// Deploys hosts of one herd.
pub struct Deployer<'a> {
    runner: &'a dyn Runner,

    manifest: &'a Manifest,

    secrets: &'a Secrets<'a>,

    /// The herd flake with the deploy-rs nodes.
    herd: &'a Path,
}

impl<'a> Deployer<'a> {
    /// A deployer for the hosts of the herd flake at `herd`.
    #[must_use]
    pub const fn new(runner: &'a dyn Runner, manifest: &'a Manifest, secrets: &'a Secrets<'a>, herd: &'a Path) -> Self {
        Self {
            runner,
            manifest,
            secrets,
            herd,
        }
    }

    /// Pushes every secret of `host` to `/var/lib/senn/secrets/<name>` over SSH.
    ///
    /// The value goes over stdin, never into arguments, so it doesn't show up in process
    /// lists. The file gets mode 0600 and is replaced atomically. One SSH connection per
    /// secret.
    ///
    /// # Errors
    ///
    /// [`crate::Error::UnknownHost`], the errors of [`Secrets::read`], and the errors of
    /// [`Runner::run`] if SSH fails.
    pub fn push_secrets(&self, host: &str) -> Result<()> {
        for (name, reference) in &self.manifest.host(host)?.secrets {
            let value = self.secrets.read(reference)?;
            // `name` is a `SecretName`, so it is safe inside this shell command.
            // umask 077 gives the file mode 0600; `mv` replaces the old file atomically.
            let file = format!("{SECRETS_DIR}/{name}");
            let tmp = format!("{SECRETS_DIR}/.{name}.tmp");
            let remote = format!(
                "umask 077 && mkdir -p {SECRETS_DIR} && chmod 700 {SECRETS_DIR} && cat > {tmp} && mv {tmp} {file}"
            );
            self.runner.run(
                &Cmd::new(&self.manifest.tools.ssh)
                    .args([format!("root@{host}"), remote])
                    .stdin(value),
            )?;
        }

        Ok(())
    }

    /// Deploys `host` with deploy-rs, or all hosts if `host` is `None`.
    ///
    /// Checks are skipped: they would build Linux closures on the operator's machine. The
    /// deploy nodes build on the host and roll back if the host becomes unreachable.
    ///
    /// # Errors
    ///
    /// The errors of [`Runner::run`] if deploy-rs fails.
    pub fn deploy(&self, host: Option<&str>) -> Result<()> {
        let target = host.map_or_else(
            || self.herd.display().to_string(),
            |h| format!("{}#{h}", self.herd.display()),
        );

        self.runner
            .run(&Cmd::new(&self.manifest.tools.deploy).args([target, "--skip-checks".to_owned()]))
            .map(drop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::tests::SAMPLE;
    use crate::runner::fake::Fake;

    #[test]
    fn pushes_each_secret_over_ssh_via_stdin() {
        let m = Manifest::parse(SAMPLE).unwrap();
        let fake = Fake::new(|cmd| match cmd.program.as_str() {
            "backend" => Some(r#"{"value":"tskey"}"#.to_owned()),
            "ssh" => Some(String::new()),
            _ => None,
        });
        let secrets = Secrets::new(&fake, "backend");

        Deployer::new(&fake, &m, &secrets, Path::new("/herd"))
            .push_secrets("cloud-a")
            .unwrap();

        let ssh = fake
            .calls()
            .into_iter()
            .find(|c| c.program == "ssh")
            .unwrap();
        assert_eq!(ssh.args[0], "root@cloud-a");
        assert_eq!(
            ssh.args[1],
            "umask 077 && mkdir -p /var/lib/senn/secrets && chmod 700 /var/lib/senn/secrets \
             && cat > /var/lib/senn/secrets/.tailscale-authkey.tmp \
             && mv /var/lib/senn/secrets/.tailscale-authkey.tmp /var/lib/senn/secrets/tailscale-authkey"
        );
        assert_eq!(ssh.stdin.as_deref(), Some("tskey"));
        assert!(!ssh.args.iter().any(|a| a.contains("tskey")), "secret leaked into args");
    }

    #[test]
    fn deploys_one_or_all_hosts() {
        let m = Manifest::parse(SAMPLE).unwrap();
        let fake = Fake::new(|_| Some(String::new()));
        let secrets = Secrets::new(&fake, "backend");
        let deployer = Deployer::new(&fake, &m, &secrets, Path::new("/herd"));

        deployer.deploy(Some("cloud-a")).unwrap();
        deployer.deploy(None).unwrap();

        let calls = fake.calls();
        assert_eq!(calls[0].program, "deploy");
        assert_eq!(calls[0].args, ["/herd#cloud-a", "--skip-checks"]);
        assert_eq!(calls[1].args, ["/herd", "--skip-checks"]);
    }

    #[test]
    fn failed_push_stops_before_the_next_secret() {
        let mut m = Manifest::parse(SAMPLE).unwrap();
        let host = m.hosts.get_mut("cloud-a").unwrap();
        host.secrets
            .insert("second".to_owned().try_into().unwrap(), "op://v/x/y".to_owned());
        let fake = Fake::new(|cmd| (cmd.program == "backend").then(|| r#"{"value":"v"}"#.to_owned()));
        let secrets = Secrets::new(&fake, "backend");

        let result = Deployer::new(&fake, &m, &secrets, Path::new("/herd")).push_secrets("cloud-a");

        assert!(matches!(result, Err(crate::Error::Failed { .. })), "{result:?}");
        assert_eq!(fake.programs().iter().filter(|p| *p == "ssh").count(), 1);
    }
}
