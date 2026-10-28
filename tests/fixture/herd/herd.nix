# The herd: everything Senn needs to know besides the hosts.
{
  senn.herd = {
    name = "fixture";
    operatorKeys = [ "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOperatorFixtureKeyOperatorFixtureKey0 operator" ];
    installKey = {
      publicKey = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIInstallFixtureKeyInstallFixtureKey00 senn-install";
      privateKey = "op://fixture/install-key/private key";
    };
    tailnet.policy = ./tailnet/policy.hujson;
    hostSecrets.tailscale-authkey = "op://fixture/tailscale/credential";
    tofu = {
      passphrase = "op://fixture/tofu/passphrase";
      env = {
        HCLOUD_TOKEN = "op://fixture/hetzner/token";
        TAILSCALE_OAUTH_CLIENT_ID = "op://fixture/tailscale-admin/username";
        TAILSCALE_OAUTH_CLIENT_SECRET = "op://fixture/tailscale-admin/credential";
      };
    };
  };
}
