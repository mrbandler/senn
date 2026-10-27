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
        (inputs.import-tree ./herd)
      ];
    };
}
