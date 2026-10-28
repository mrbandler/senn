# The herd: everything Senn needs to know besides the hosts.
# Values starting with op:// are 1Password references, resolved at apply time.
{
  senn.herd = {
    name = "my-herd";
    operatorKeys = [ "<your SSH public key>" ];
    installKey = {
      publicKey = "<public key of the install-key item>";
      privateKey = "op://senn-infra/install-key/private key?ssh-format=openssh";
    };
    # Senn manages the tailnet policy from this file and keeps MagicDNS on.
    tailnet.policy = ./tailnet/policy.hujson;
    hostSecrets.tailscale-authkey = "op://senn-infra/tailscale/credential";
    tofu = {
      passphrase = "op://senn-infra/tofu/password";
      env = {
        HCLOUD_TOKEN = "op://senn-infra/hetzner/credential";
        # An OAuth credential with the scopes policy_file and dns (write).
        TAILSCALE_OAUTH_CLIENT_ID = "op://senn-infra/tailscale-admin/username";
        TAILSCALE_OAUTH_CLIENT_SECRET = "op://senn-infra/tailscale-admin/credential";
      };
    };
  };
}
