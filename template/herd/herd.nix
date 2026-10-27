# The herd: everything Senn needs to know besides the hosts.
# Values starting with op:// are 1Password references, resolved at apply time.
{
  senn.herd = {
    name = "my-herd";
    operatorKeys = [ "<your SSH public key>" ];
    installKey = {
      publicKey = "<public key of the install-key item>";
      privateKey = "op://senn-infra/install-key/private key";
    };
    hostSecrets.tailscale-authkey = "op://senn-infra/tailscale/credential";
    tofu = {
      passphrase = "op://senn-infra/tofu/passphrase";
      env.HCLOUD_TOKEN = "op://senn-infra/hetzner/token";
    };
  };
}
