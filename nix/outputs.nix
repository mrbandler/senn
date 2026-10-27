# Everything the CLI consumes: deploy nodes, the tofu config and the manifest.
{ inputs, ... }:
{ config, lib, ... }:
let
  cfg = config.senn.herd;
  hosts = lib.foldl' (acc: perSystem: acc // perSystem) { } (lib.attrValues config.den.hosts);
  generated = import ./tofu.nix {
    inherit lib hosts;
    herd = cfg;
  };
in
{
  # den derives flake-parts `systems` from this; its default (only the hosts' systems)
  # would leave no manifest for the operator's machine.
  den.systems = lib.mkDefault [
    "x86_64-linux"
    "aarch64-linux"
    "x86_64-darwin"
    "aarch64-darwin"
  ];

  flake.deploy.nodes = lib.mapAttrs (name: host: {
    hostname = name; # MagicDNS name on the tailnet
    sshUser = "root";
    remoteBuild = true; # the operator's machine may not build Linux
    magicRollback = true;
    profiles.system = {
      user = "root";
      path = inputs.deploy-rs.lib.${host.system}.activate.nixos config.flake.nixosConfigurations.${name};
    };
  }) hosts;

  perSystem =
    { pkgs, system, ... }:
    let
      tofu = pkgs.opentofu.withPlugins generated.nixpkgsProviders;
      core = pkgs.writeText "main.tf.json" (builtins.toJSON generated.config);
    in
    {
      packages.senn-core = core;

      packages.senn-manifest = pkgs.writeText "senn-manifest.json" (
        builtins.toJSON {
          herd = cfg.name;
          tools = {
            tofu = "${tofu}/bin/tofu";
            nixosAnywhere = "${pkgs.nixos-anywhere}/bin/nixos-anywhere";
            deploy = "${inputs.deploy-rs.packages.${system}.default}/bin/deploy";
            backend = lib.getExe (cfg.secrets.backend pkgs);
            ssh = "${pkgs.openssh}/bin/ssh";
          };
          core = "${core}";
          tofu = { inherit (cfg.tofu) passphrase env; };
          installKey = cfg.installKey.privateKey;
          hosts = lib.mapAttrs (_: host: { secrets = cfg.hostSecrets // host.secrets; }) hosts;
        }
      );

      checks.senn-core = pkgs.runCommand "senn-core-validate" { nativeBuildInputs = [ tofu ]; } ''
        export HOME=$TMPDIR
        cp ${core} main.tf.json
        tofu init -backend=false -input=false
        tofu validate
        touch $out
      '';
    };
}
