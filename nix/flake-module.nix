# Entry point of Senn for instance flakes (flake-parts module).
# Called with Senn's own inputs, so instances only need `senn` as an input.
{ inputs, self }:
{
  imports = [ inputs.den.flakeModule ];
}
