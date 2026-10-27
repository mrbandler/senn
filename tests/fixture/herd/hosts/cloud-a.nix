{ ... }:
{
  den.hosts.x86_64-linux.cloud-a = {
    provision = {
      kind = "stub";
    };
  };
  den.aspects.cloud-a = { };
}
