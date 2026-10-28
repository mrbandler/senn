//! The manifest: everything the CLI needs from a herd, built by Nix as `.#senn-manifest`.
//!
//! It holds tool paths and secret references, never secret values. The Nix side is
//! `nix/outputs.nix`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{Error, Result};
use crate::runner::{Cmd, Runner, run};

/// The manifest of one herd.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    /// Name of the herd (`senn.herd.name`).
    pub herd: String,

    /// Store paths of the tools, pinned by the herd's `flake.lock`.
    pub tools: Tools,

    /// The generated OpenTofu config of the core root (`main.tf.json`).
    pub core: PathBuf,

    /// Secrets OpenTofu needs.
    pub tofu: TofuSecrets,

    /// Secret reference to the private half of the install key.
    pub install_key: String,

    /// Hosts by name.
    pub hosts: BTreeMap<String, Host>,
}

/// Paths of the programs Senn runs.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tools {
    /// OpenTofu, with the providers of the herd.
    pub tofu: String,

    /// nixos-anywhere, for the first install.
    pub nixos_anywhere: String,

    /// deploy-rs, for updates.
    pub deploy: String,

    /// The secrets backend (see [`crate::protocol`]).
    pub backend: String,

    /// OpenSSH client.
    pub ssh: String,
}

/// Secret references for OpenTofu.
#[derive(Debug, Clone, Deserialize)]
pub struct TofuSecrets {
    /// Reference to the passphrase for state encryption.
    pub passphrase: String,

    /// Environment variable name to secret reference, for providers (for example `HCLOUD_TOKEN`).
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

/// One host of the herd.
#[derive(Debug, Clone, Deserialize)]
pub struct Host {
    /// File name in `/var/lib/senn/secrets` to secret reference.
    #[serde(default)]
    pub secrets: BTreeMap<String, String>,
}

impl Manifest {
    /// Parses a manifest from JSON.
    ///
    /// # Errors
    ///
    /// [`Error::Json`] if the JSON doesn't match the manifest format.
    pub fn parse(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Builds `.#senn-manifest` in the herd flake at `herd` and reads it.
    ///
    /// # Errors
    ///
    /// The errors of [`run`] if `nix build` fails, [`Error::Io`] if the result can't be
    /// read, and the errors of [`Manifest::parse`].
    pub fn load(runner: &dyn Runner, herd: &Path) -> Result<Self> {
        let path = run(
            runner,
            &Cmd::new("nix")
                .args(["build", ".#senn-manifest", "--no-link", "--print-out-paths"])
                .cwd(herd)
                .capture(),
        )?;
        Self::parse(&std::fs::read_to_string(path.trim())?)
    }

    /// The host called `name`.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownHost`] if the herd has no such host.
    pub fn host(&self, name: &str) -> Result<&Host> {
        self.hosts
            .get(name)
            .ok_or_else(|| Error::UnknownHost(name.to_owned()))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A manifest in the format `nix/outputs.nix` produces.
    pub const SAMPLE: &str = r#"{
      "herd": "fixture",
      "tools": { "tofu": "tofu", "nixosAnywhere": "nixos-anywhere", "deploy": "deploy", "backend": "backend", "ssh": "ssh" },
      "core": "/tmp/main.tf.json",
      "tofu": { "passphrase": "op://v/tofu/passphrase", "env": { "HCLOUD_TOKEN": "op://v/hetzner/token" } },
      "installKey": "op://v/install/key",
      "hosts": { "cloud-a": { "secrets": { "tailscale-authkey": "op://v/ts/key" } } }
    }"#;

    #[test]
    fn parses_the_nix_manifest() {
        let m = Manifest::parse(SAMPLE).unwrap();
        assert_eq!(m.herd, "fixture");
        assert_eq!(m.tools.nixos_anywhere, "nixos-anywhere");
        assert_eq!(m.tofu.env["HCLOUD_TOKEN"], "op://v/hetzner/token");
        assert_eq!(m.host("cloud-a").unwrap().secrets["tailscale-authkey"], "op://v/ts/key");
    }

    #[test]
    fn unknown_host_is_an_error() {
        let m = Manifest::parse(SAMPLE).unwrap();
        assert!(matches!(m.host("nope"), Err(Error::UnknownHost(h)) if h == "nope"));
    }

    #[test]
    fn load_builds_the_manifest_with_nix() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("manifest.json");
        std::fs::write(&file, SAMPLE).unwrap();
        let path = file.display().to_string();
        let fake = crate::runner::fake::Fake::new(move |_| Some(format!("{path}\n")));

        let m = Manifest::load(&fake, Path::new("/herd")).unwrap();

        assert_eq!(m.herd, "fixture");
        let call = &fake.calls()[0];
        assert_eq!(call.program, "nix");
        assert_eq!(
            call.args,
            ["build", ".#senn-manifest", "--no-link", "--print-out-paths"]
        );
        assert_eq!(call.cwd.as_deref(), Some(Path::new("/herd")));
    }
}
