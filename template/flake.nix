{
  inputs = {
    senn.url = "https://flakehub.com/f/mrbandler/senn/0.3.*";
    # Senn's den module reads these from the herd's own inputs.
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
        # Every .nix file under herd/ is loaded; prefix helpers with `_` to skip them.
        (inputs.import-tree ./herd)
      ];
    };
}
