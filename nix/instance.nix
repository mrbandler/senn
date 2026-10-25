# Options for one Senn instance, and the fields a host definition carries.
{ self, ... }:
{ lib, ... }:
let
  inherit (lib) mkOption types;
  ref = types.str; # a secret reference, resolved by the secrets backend
in
{
  options.senn.instance = {
    name = mkOption {
      type = types.str;
      description = "Instance name.";
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
    tailnet.tags = mkOption {
      type = types.listOf types.str;
      default = [ "tag:server" ];
      description = "Tailscale tags every host advertises.";
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
