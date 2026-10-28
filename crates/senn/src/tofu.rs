//! Runs OpenTofu on the generated core root in `<herd>/.senn/core`.
//!
//! Provider secrets go into the environment of each `tofu` call, never into files. State
//! and plan files are encrypted with a key derived from the herd's tofu passphrase.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{Error, Result};
use crate::manifest::Manifest;
use crate::runner::{Cmd, Runner};
use crate::secrets::Secrets;

/// OpenTofu, ready to run in the core root of one herd.
pub struct Tofu<'a> {
    runner: &'a dyn Runner,

    program: String,

    dir: PathBuf,

    /// Resolved provider secrets and `TF_ENCRYPTION`. Never log these.
    env: Vec<(String, String)>,
}

/// The addresses of one host, from the `hosts` output of the core root.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct HostAddr {
    /// Public IPv4 address.
    pub ipv4: String,

    /// Public IPv6 address.
    pub ipv6: String,
}

impl<'a> Tofu<'a> {
    /// Writes the generated config to `<herd>/.senn/core/main.tf.json` and resolves the
    /// provider secrets and the state passphrase through `secrets`.
    ///
    /// # Errors
    ///
    /// [`Error::Io`] if the directory or the config can't be written, and the errors of
    /// [`Secrets::read`].
    pub fn prepare(runner: &'a dyn Runner, manifest: &Manifest, secrets: &Secrets, herd: &Path) -> Result<Self> {
        let dir = herd.join(".senn").join("core");
        std::fs::create_dir_all(&dir)?;
        // The store copy is read-only; write a fresh file so tofu can replace it later.
        std::fs::write(dir.join("main.tf.json"), std::fs::read(&manifest.core)?)?;

        let mut env = Vec::with_capacity(manifest.tofu.env.len() + 1);
        for (name, reference) in &manifest.tofu.env {
            env.push((name.clone(), secrets.read(reference)?));
        }
        let passphrase = secrets.read(&manifest.tofu.passphrase)?;
        env.push(("TF_ENCRYPTION".to_owned(), Self::encryption_config(&passphrase)));

        Ok(Self {
            runner,
            program: manifest.tools.tofu.clone(),
            dir,
            env,
        })
    }

    fn cmd(&self, args: &[&str]) -> Cmd {
        let mut cmd = Cmd::new(&self.program)
            .args(args.iter().copied())
            .cwd(&self.dir);
        for (key, value) in &self.env {
            cmd = cmd.env(key, value);
        }
        cmd
    }

    /// Runs `tofu init`.
    ///
    /// # Errors
    ///
    /// The errors of [`Runner::run`].
    pub fn init(&self) -> Result<()> {
        self.runner
            .run(&self.cmd(&["init", "-input=false"]))
            .map(drop)
    }

    /// Plans into `senn.plan` and returns whether there are changes.
    ///
    /// # Errors
    ///
    /// The errors of [`Runner::exec`], and [`Error::Failed`] if tofu exits with a code
    /// other than 0 (no changes) or 2 (changes).
    pub fn plan(&self) -> Result<bool> {
        let cmd = self.cmd(&["plan", "-input=false", "-detailed-exitcode", "-out=senn.plan"]);
        match self.runner.exec(&cmd)?.code {
            Some(0) => Ok(false),
            Some(2) => Ok(true),
            code => Err(Error::Failed {
                program: self.program.clone(),
                code,
            }),
        }
    }

    /// Applies `senn.plan`, exactly what [`Tofu::plan`] showed.
    ///
    /// # Errors
    ///
    /// The errors of [`Runner::run`].
    pub fn apply(&self) -> Result<()> {
        self.runner
            .run(&self.cmd(&["apply", "-input=false", "senn.plan"]))
            .map(drop)
    }

    /// Reads the `hosts` output: host name to addresses.
    ///
    /// # Errors
    ///
    /// The errors of [`Runner::run`], and [`Error::Json`] if the output has another shape.
    pub fn hosts(&self) -> Result<BTreeMap<String, HostAddr>> {
        let out = self
            .runner
            .run(&self.cmd(&["output", "-json", "hosts"]).capture())?;
        Ok(serde_json::from_str(&out)?)
    }

    /// Builds the `TF_ENCRYPTION` value: state and plan encrypted with AES-GCM and a PBKDF2
    /// key derived from `passphrase`. `enforced` stops tofu from ever writing them unencrypted.
    ///
    /// The passphrase is escaped for an HCL string, including the template sequences `${`
    /// and `%{`. OpenTofu requires at least 16 characters.
    fn encryption_config(passphrase: &str) -> String {
        let escaped = passphrase
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace("${", "$${")
            .replace("%{", "%%{");
        format!(
            r#"key_provider "pbkdf2" "senn" {{
      passphrase = "{escaped}"
    }}
    method "aes_gcm" "senn" {{
      keys = key_provider.pbkdf2.senn
    }}
    state {{
      method   = method.aes_gcm.senn
      enforced = true
    }}
    plan {{
      method   = method.aes_gcm.senn
      enforced = true
    }}
    "#
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::tests::SAMPLE;
    use crate::runner::Output;
    use crate::runner::fake::Fake;

    fn manifest_with_core(dir: &Path) -> Manifest {
        let core = dir.join("core.json");
        std::fs::write(&core, r#"{"resource":{}}"#).unwrap();
        let mut m = Manifest::parse(SAMPLE).unwrap();
        m.core = core;
        m
    }

    fn respond(cmd: &Cmd) -> Option<String> {
        match (cmd.program.as_str(), cmd.args.first().map(String::as_str)) {
            ("backend", _) => Some(r#"{"value":"passphrase-0123456789"}"#.to_owned()),
            ("tofu", Some("output")) => Some(r#"{"cloud-a":{"ipv4":"1.2.3.4","ipv6":"2a01::1"}}"#.to_owned()),
            ("tofu", _) => Some(String::new()),
            _ => None,
        }
    }

    #[test]
    fn prepare_writes_config_and_resolves_secrets_into_env() {
        let dir = tempfile::tempdir().unwrap();
        let m = manifest_with_core(dir.path());
        let fake = Fake::new(respond);
        let secrets = Secrets::new(&fake, "backend");

        let tofu = Tofu::prepare(&fake, &m, &secrets, dir.path()).unwrap();
        tofu.init().unwrap();

        let core = dir.path().join(".senn").join("core");
        let written = std::fs::read_to_string(core.join("main.tf.json")).unwrap();
        assert_eq!(written, r#"{"resource":{}}"#);
        let init = fake
            .calls()
            .into_iter()
            .find(|c| c.program == "tofu")
            .unwrap();
        assert_eq!(init.args, ["init", "-input=false"]);
        assert_eq!(init.cwd.as_deref(), Some(core.as_path()));
        let env: BTreeMap<_, _> = init.env.into_iter().collect();
        assert_eq!(env["HCLOUD_TOKEN"], "passphrase-0123456789");
        assert!(env["TF_ENCRYPTION"].contains("passphrase-0123456789"));
    }

    #[test]
    fn plan_reports_changes_from_the_detailed_exit_code() {
        let dir = tempfile::tempdir().unwrap();
        let m = manifest_with_core(dir.path());
        let code = std::rc::Rc::new(std::cell::Cell::new(2));
        let c = code.clone();
        let fake = FakeCode::new(move |cmd| if cmd.program == "tofu" { c.get() } else { 0 });
        let secrets = Secrets::new(&fake, "backend");
        let tofu = Tofu::prepare(&fake, &m, &secrets, dir.path()).unwrap();

        assert!(tofu.plan().unwrap());
        code.set(0);
        assert!(!tofu.plan().unwrap());
        code.set(1);
        assert!(tofu.plan().is_err());
    }

    #[test]
    fn hosts_parses_the_output() {
        let dir = tempfile::tempdir().unwrap();
        let m = manifest_with_core(dir.path());
        let fake = Fake::new(respond);
        let secrets = Secrets::new(&fake, "backend");

        let hosts = Tofu::prepare(&fake, &m, &secrets, dir.path())
            .unwrap()
            .hosts()
            .unwrap();

        assert_eq!(hosts["cloud-a"].ipv4, "1.2.3.4");
        assert_eq!(hosts["cloud-a"].ipv6, "2a01::1");
    }

    #[test]
    fn encryption_config_escapes_the_passphrase() {
        let hcl = Tofu::encryption_config(r#"a"b\c${d}%{e}"#);
        assert!(hcl.contains(r#"passphrase = "a\"b\\c$${d}%%{e}""#), "{hcl}");
        assert!(hcl.contains("enforced = true"), "{hcl}");
    }

    /// Like `Fake`, but answers with an exit code. Backend calls get a valid response.
    struct FakeCode(Box<dyn Fn(&Cmd) -> i32>);

    impl FakeCode {
        fn new(f: impl Fn(&Cmd) -> i32 + 'static) -> Self {
            Self(Box::new(f))
        }
    }

    impl Runner for FakeCode {
        fn exec(&self, cmd: &Cmd) -> Result<Output> {
            let stdout = if cmd.program == "backend" {
                r#"{"value":"v"}"#.to_owned()
            } else {
                String::new()
            };
            Ok(Output {
                code: Some((self.0)(cmd)),
                stdout,
            })
        }
    }
}
