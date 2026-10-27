{
  description = "senn: Git-native infrastructure on Nix and OpenTofu";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    import-tree.url = "github:denful/import-tree";
    den.url = "github:denful/den/v0.19.0";
    srvos = {
      url = "github:nix-community/srvos";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    disko = {
      url = "github:nix-community/disko/v1.13.0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    deploy-rs = {
      url = "github:serokell/deploy-rs/v1.0.0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      ...
    }@inputs:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f (
            import nixpkgs {
              inherit system;
              overlays = [ (import rust-overlay) ];
              # The 1Password backend needs the unfree `op` CLI.
              config.allowUnfreePredicate = pkg: nixpkgs.lib.getName pkg == "1password-cli";
            }
          )
        );
      # Version comes from Cargo.toml (single source of truth).
      cargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    in
    {
      flakeModules.default = import ./nix/flake-module.nix { inherit inputs self; };

      lib.presets.hetzner = import ./nix/presets/hetzner.nix { inherit inputs; };

      packages = forAllSystems (
        pkgs:
        let
          # Same toolchain as rustup and devenv.
          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          };
        in
        {
          default = rustPlatform.buildRustPackage {
            pname = "senn";
            version = cargo.workspace.package.version;
            src = pkgs.lib.cleanSource ./.;
            # Dependency hashes come from Cargo.lock; nothing to update by hand.
            cargoLock.lockFile = ./Cargo.lock;
            cargoBuildFlags = [ "--package" "senn-cli" ];
            cargoTestFlags = [ "--package" "senn-cli" ];
            meta.mainProgram = "senn";
          };
        }
      );
    };
}
