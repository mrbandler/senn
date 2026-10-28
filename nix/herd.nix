# Options for one Senn herd (one infra project), and the fields a host definition carries.
{ self, ... }:
{ lib, ... }:
let
  inherit (lib) mkOption types;
  ref = types.str; # a secret reference, resolved by the secrets backend
in
{
  options.senn.herd = {
    name = mkOption {
      type = types.str;
      description = "Herd name.";
    };
    operatorKeys = mkOption {
      type = types.listOf types.str;
      description = "SSH public keys allowed as root over the tailnet (break-glass and laptop deploys).";
    };
    installKey = {
      publicKey = mkOption {
        type = types.str;
        description = "Install key, public half. Handed to provisioning so fresh machines accept the install.";
      };
      privateKey = mkOption {
        type = ref;
        description = "Secret reference to the install key's private half.";
      };
    };
    tailnet = {
      tags = mkOption {
        type = types.listOf types.str;
        default = [ "tag:server" ];
        description = "Tailscale tags every host advertises.";
      };
      policy = mkOption {
        type = types.nullOr types.path;
        default = null;
        description = ''
          The tailnet policy file (HuJSON), kept in the herd repo. When set, Senn manages the
          tailnet's policy from it and keeps MagicDNS on. On the first apply, the current policy
          is imported, so put your existing policy into this file first. The Tailscale provider
          logs in through TAILSCALE_OAUTH_CLIENT_ID and TAILSCALE_OAUTH_CLIENT_SECRET in
          `tofu.env`; the credential needs the scopes policy_file and dns (write).
        '';
      };
    };
    secrets.backend = mkOption {
      type = types.functionTo types.package;
      default = pkgs: self.packages.${pkgs.stdenv.hostPlatform.system}.senn-backend-onepassword;
      description = "Secrets backend program (protocol 1), per system.";
    };
    hostSecrets = mkOption {
      type = types.attrsOf ref;
      default = { };
      description = "Secrets pushed to every host: file name in /var/lib/senn/secrets -> secret reference.";
    };
    tofu = {
      passphrase = mkOption {
        type = ref;
        description = "Secret reference to the state encryption passphrase (16+ characters).";
      };
      env = mkOption {
        type = types.attrsOf ref;
        default = { };
        description = "Environment for tofu: variable name -> secret reference (e.g. HCLOUD_TOKEN).";
      };
    };
  };

  config.den.schema.host.imports = [
    (
      { lib, ... }:
      {
        options.provision = lib.mkOption {
          type = lib.types.attrs;
          description = "Where the machine comes from: the result of a preset.";
        };
        options.secrets = lib.mkOption {
          type = lib.types.attrsOf lib.types.str;
          default = { };
          description = "Extra secrets for this host: file name -> secret reference.";
        };
      }
    )
  ];
}
