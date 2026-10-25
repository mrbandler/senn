# Entry point of Senn for instance flakes (flake-parts module).
# Called with Senn's own inputs. den reads `den`, `nixpkgs` and `import-tree`
# from the instance's inputs, so instances follow those from `senn`.
{ inputs, self }:
{
  imports = [
    inputs.den.flakeModule
    (import ./instance.nix { inherit inputs self; })
  ];
}
