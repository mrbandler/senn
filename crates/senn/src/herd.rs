//! The `senn` workflows on one herd: plan, apply (create, install, configure) and deploy.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::deployer::Deployer;
use crate::error::{Error, Result};
use crate::installer::Installer;
use crate::manifest::Manifest;
use crate::runner::Runner;
use crate::secrets::Secrets;
use crate::tofu::Tofu;

/// How long a fresh host may take to reboot and join the tailnet after its install.
const JOIN_TIMEOUT: Duration = Duration::from_secs(600);

/// How often to probe a fresh host while waiting for it.
const JOIN_INTERVAL: Duration = Duration::from_secs(10);

/// One herd: its repository and its manifest.
pub struct Herd<'a> {
    runner: &'a dyn Runner,

    /// The herd repository, the root of its flake.
    path: PathBuf,

    manifest: Manifest,
}

impl<'a> Herd<'a> {
    /// Loads the herd at `path` by building its manifest with Nix.
    ///
    /// # Errors
    ///
    /// The errors of [`Manifest::load`].
    pub fn load(runner: &'a dyn Runner, path: &Path) -> Result<Self> {
        Ok(Self {
            runner,
            path: path.to_owned(),
            manifest: Manifest::load(runner, path)?,
        })
    }

    /// Shows what would change in the core root, without changing anything.
    ///
    /// # Errors
    ///
    /// The errors of [`Tofu::prepare`], [`Tofu::init`] and [`Tofu::plan`].
    pub fn plan(&self) -> Result<()> {
        let secrets = self.secrets();
        let tofu = Tofu::prepare(self.runner, &self.manifest, &secrets, &self.path)?;
        tofu.init()?;

        tofu.plan().map(drop)
    }

    /// Brings every host to its declared state, in three stages:
    ///
    /// 1. Create: plan the core root and, if there are changes and `confirm` returns
    ///    `true`, apply exactly that plan.
    /// 2. Install: install every host that is not on the tailnet yet, and wait until it is.
    /// 3. Configure: push the secrets of every host, then deploy all hosts.
    ///
    /// # Errors
    ///
    /// [`Error::Aborted`] if `confirm` returns `false`, before anything changes. Otherwise
    /// the errors of the stages; a failed stage stops the ones after it.
    pub fn apply(&self, confirm: &dyn Fn() -> Result<bool>) -> Result<()> {
        let secrets = self.secrets();

        let tofu = Tofu::prepare(self.runner, &self.manifest, &secrets, &self.path)?;
        tofu.init()?;
        if tofu.plan()? {
            if !confirm()? {
                return Err(Error::Aborted);
            }
            tofu.apply()?;
        }
        let hosts = tofu.hosts()?;

        let installer = Installer::new(self.runner, &self.manifest, &secrets, &self.path);
        for (name, addr) in &hosts {
            if !installer.is_installed(name)? {
                installer.install(name, &addr.ipv4)?;
                installer.wait_until_installed(name, JOIN_TIMEOUT, JOIN_INTERVAL)?;
            }
        }

        let deployer = Deployer::new(self.runner, &self.manifest, &secrets, &self.path);
        for name in hosts.keys() {
            deployer.push_secrets(name)?;
        }

        deployer.deploy(None)
    }

    /// Pushes secrets and deploys `host`, or all hosts if `host` is `None`. No OpenTofu.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownHost`], and the errors of [`Deployer::push_secrets`] and
    /// [`Deployer::deploy`].
    pub fn deploy(&self, host: Option<&str>) -> Result<()> {
        let secrets = self.secrets();
        let deployer = Deployer::new(self.runner, &self.manifest, &secrets, &self.path);

        let names: Vec<&str> =
            host.map_or_else(|| self.manifest.hosts.keys().map(String::as_str).collect(), |h| vec![h]);
        for name in names {
            deployer.push_secrets(name)?;
        }

        deployer.deploy(host)
    }

    fn secrets(&self) -> Secrets<'_> {
        Secrets::new(self.runner, &self.manifest.tools.backend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::tests::SAMPLE;
    use crate::runner::fake::Fake;
    use crate::runner::{Cmd, Output};

    /// A fake world: "nix" builds the manifest; the host joins the tailnet once installed.
    fn world(dir: &Path) -> Fake {
        let core = dir.join("core.json");
        std::fs::write(&core, "{}").unwrap();
        let manifest = dir.join("manifest.json");
        let json = SAMPLE.replace("/tmp/main.tf.json", &core.display().to_string().replace('\\', "\\\\"));
        std::fs::write(&manifest, json).unwrap();
        let manifest = manifest.display().to_string();
        let installed = std::rc::Rc::new(std::cell::Cell::new(false));

        Fake::new(
            move |cmd| match (cmd.program.as_str(), cmd.args.first().map(String::as_str)) {
                ("nix", _) => Some(manifest.clone()),
                ("backend", _) => Some(r#"{"value":"v-0123456789abcdef"}"#.to_owned()),
                ("tofu", Some("output")) => Some(r#"{"cloud-a":{"ipv4":"1.2.3.4","ipv6":"2a01::1"}}"#.to_owned()),
                // The probe answers only after nixos-anywhere ran.
                ("ssh", _) if cmd.args.last().is_some_and(|a| a == "true") => installed.get().then(String::new),
                ("nixos-anywhere", _) => {
                    installed.set(true);
                    Some(String::new())
                },
                ("tofu" | "deploy" | "ssh", _) => Some(String::new()),
                _ => None,
            },
        )
    }

    fn steps(fake: &Fake) -> Vec<String> {
        fake.calls()
            .into_iter()
            .filter(|c| c.program != "backend")
            .map(|c| match c.program.as_str() {
                "tofu" => format!("tofu {}", c.args[0]),
                "ssh" if c.args.last().is_some_and(|a| a == "true") => "probe".to_owned(),
                "ssh" => "push".to_owned(),
                other => other.to_owned(),
            })
            .collect()
    }

    #[test]
    fn apply_runs_create_install_configure_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let fake = ChangesFake(world(dir.path()));

        Herd::load(&fake, dir.path())
            .unwrap()
            .apply(&|| Ok(true))
            .unwrap();

        assert_eq!(
            steps(&fake.0),
            [
                "nix",
                "tofu init",
                "tofu plan",
                "tofu apply",
                "tofu output",
                "probe",
                "nixos-anywhere",
                "probe",
                "push",
                "deploy"
            ]
        );
    }

    #[test]
    fn declining_the_plan_aborts_before_anything_changes() {
        let dir = tempfile::tempdir().unwrap();
        let fake = ChangesFake(world(dir.path()));

        let result = Herd::load(&fake, dir.path()).unwrap().apply(&|| Ok(false));

        assert!(matches!(result, Err(Error::Aborted)), "{result:?}");
        assert_eq!(steps(&fake.0), ["nix", "tofu init", "tofu plan"]);
    }

    #[test]
    fn no_changes_skips_confirmation_and_apply() {
        let dir = tempfile::tempdir().unwrap();
        let fake = world(dir.path());

        Herd::load(&fake, dir.path())
            .unwrap()
            .apply(&|| panic!("must not ask"))
            .unwrap();

        assert!(!steps(&fake).contains(&"tofu apply".to_owned()));
    }

    #[test]
    fn deploy_one_host_pushes_only_its_secrets() {
        let dir = tempfile::tempdir().unwrap();
        let fake = world(dir.path());

        Herd::load(&fake, dir.path())
            .unwrap()
            .deploy(Some("cloud-a"))
            .unwrap();

        assert_eq!(steps(&fake), ["nix", "push", "deploy"]);
        let result = Herd::load(&fake, dir.path()).unwrap().deploy(Some("nope"));
        assert!(matches!(result, Err(Error::UnknownHost(_))), "{result:?}");
    }

    /// Makes `tofu plan` exit 2 (changes); everything else as the inner fake.
    struct ChangesFake(Fake);

    impl Runner for ChangesFake {
        fn exec(&self, cmd: &Cmd) -> Result<Output> {
            let mut out = self.0.exec(cmd)?;
            if cmd.program == "tofu" && cmd.args.first().is_some_and(|a| a == "plan") {
                out.code = Some(2);
            }
            Ok(out)
        }
    }
}
