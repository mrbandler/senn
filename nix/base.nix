# The `base` aspect: what Senn needs to manage a host at all. Every host gets it.
{ inputs, ... }:
{ config, den, ... }:
let
  cfg = config.senn.herd;
in
{
  den.aspects.base =
    { host, ... }:
    {
      nixos =
        { lib, ... }:
        {
          imports = [ inputs.srvos.nixosModules.server ] ++ lib.optional (host.provision ? nixos) host.provision.nixos;

          networking.hostName = host.hostName;
          system.stateVersion = lib.mkDefault "26.05";

          # Reached only over the tailnet: no public SSH, port 22 on tailscale0 only.
          services.openssh.openFirewall = lib.mkForce false;
          networking.firewall.enable = true;
          networking.firewall.interfaces.tailscale0.allowedTCPPorts = [ 22 ];
          users.users.root.openssh.authorizedKeys.keys = cfg.operatorKeys;

          # Senn pushes host secrets here (install: nixos-anywhere --extra-files, later: over the tailnet).
          systemd.tmpfiles.rules = [ "d /var/lib/senn/secrets 0700 root root -" ];

          services.tailscale = {
            enable = true;
            authKeyFile = "/var/lib/senn/secrets/tailscale-authkey";
            authKeyParameters = {
              ephemeral = false;
              preauthorized = true;
            };
            extraUpFlags = [ "--advertise-tags=${lib.concatStringsSep "," cfg.tailnet.tags}" ];
          };
        };
    };

  den.default.includes = [ den.aspects.base ];
}
