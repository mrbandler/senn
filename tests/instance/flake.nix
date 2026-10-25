{
  inputs = {
    senn.url = "path:../..";
    nixpkgs.follows = "senn/nixpkgs";
    flake-parts.follows = "senn/flake-parts";
    import-tree.follows = "senn/import-tree";
    den.follows = "senn/den";
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.senn.flakeModules.default
        (inputs.import-tree ./hosts)
      ];

      senn.instance = {
        name = "fixture";
        operatorKeys = [ "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOperatorFixtureKeyOperatorFixtureKey0 operator" ];
        installKey = {
          publicKey = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIInstallFixtureKeyInstallFixtureKey00 senn-install";
          privateKey = "op://fixture/install-key/private key";
        };
        hostSecrets.tailscale-authkey = "op://fixture/tailscale/credential";
        tofu = {
          passphrase = "op://fixture/tofu/passphrase";
          env.HCLOUD_TOKEN = "op://fixture/hetzner/token";
        };
      };
    };
}
