# One host: where it runs (provision) and what runs on it (the aspect).
{ inputs, ... }:
{
  den.hosts.x86_64-linux.example = {
    provision = inputs.senn.lib.presets.hetzner {
      type = "cpx22";
      location = "nbg1";
    };
  };

  den.aspects.example.nixos = {
    # NixOS configuration for this host.
  };
}
