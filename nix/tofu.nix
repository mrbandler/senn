# Pure generator: host definitions -> tofu config (core root) as a Nix attrset.
{
  lib,
  herd,
  hosts,
}:
let
  hostList = lib.mapAttrsToList (name: host: {
    inherit name;
    p = host.provision;
  }) hosts;
  args = {
    installPublicKey = herd.installKey.publicKey;
    herdName = herd.name;
  };
  perHost = map (h: h.p.tofu (args // { inherit (h) name; })) hostList;
  kinds = lib.unique (map (h: h.p.kind) hostList);
  presetOf = kind: (lib.findFirst (h: h.p.kind == kind) null hostList).p;
  # The tailnet: its policy file and MagicDNS, when the herd manages it.
  managesTailnet = herd.tailnet.policy != null;
  tailnet = lib.optionalAttrs managesTailnet {
    # No version constraint: the provider comes from nixpkgs, so flake.lock is the only pin.
    terraform.required_providers.tailscale.source = "tailscale/tailscale";
    resource.tailscale_acl.tailnet.acl = builtins.readFile herd.tailnet.policy;
    # Senn reaches hosts by their MagicDNS name.
    resource.tailscale_dns_preferences.tailnet.magic_dns = true;
  };
  # Adopts the tailnet's current settings on the first apply; a no-op once they are in state.
  tailnetImports = lib.optionals managesTailnet [
    {
      to = "tailscale_acl.tailnet";
      id = "acl";
    }
    {
      to = "tailscale_dns_preferences.tailnet";
      id = "dns_preferences";
    }
  ];
  imports = lib.concatMap (t: t.imports) perHost ++ tailnetImports;
in
{
  nixpkgsProviders =
    p:
    lib.concatMap (kind: (presetOf kind).nixpkgsProviders p) kinds
    ++ lib.optional managesTailnet p.tailscale_tailscale;

  config =
    lib.foldl' lib.recursiveUpdate { } (
      map (kind: (presetOf kind).shared args) kinds
      ++ map (t: t.config) perHost
      ++ [
        tailnet
        {
          terraform.required_providers = lib.foldl' (acc: kind: acc // (presetOf kind).providers) { } kinds;
          output.hosts.value = lib.listToAttrs (
            lib.zipListsWith (h: t: lib.nameValuePair h.name { inherit (t) ipv4 ipv6; }) hostList perHost
          );
        }
      ]
    )
    // lib.optionalAttrs (imports != [ ]) { import = imports; };
}
