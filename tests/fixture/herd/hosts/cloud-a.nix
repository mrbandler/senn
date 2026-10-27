{ inputs, ... }:
{
  den.hosts.x86_64-linux.cloud-a = {
    provision = inputs.senn.lib.presets.hetzner {
      type = "cpx32";
      location = "nbg1";
      adopt = 123456;
    };
  };
  den.aspects.cloud-a = { };
}
