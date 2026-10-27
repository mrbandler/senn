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
  imports = lib.concatMap (t: t.imports) perHost;
in
{
  nixpkgsProviders = p: lib.concatMap (kind: (presetOf kind).nixpkgsProviders p) kinds;

  config =
    lib.foldl' lib.recursiveUpdate { } (
      map (kind: (presetOf kind).shared args) kinds
      ++ map (t: t.config) perHost
      ++ [
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
