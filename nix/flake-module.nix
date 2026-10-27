# Entry point of Senn for herd flakes (flake-parts module).
# Called with Senn's own inputs. den reads `den`, `nixpkgs` and `import-tree`
# from the herd's inputs, so herds follow those from `senn`.
{ inputs, self }:
{
  imports = [
    inputs.den.flakeModule
    (import ./herd.nix { inherit inputs self; })
  ];
}
