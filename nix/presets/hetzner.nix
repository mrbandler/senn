# Hetzner Cloud preset: creates (or adopts) one server and gives it the install key.
{ inputs }:
{
  type,
  location,
  image ? "ubuntu-24.04",
  adopt ? null,
}:
{
  kind = "hetzner";

  nixpkgsProviders = p: [ p.hetznercloud_hcloud ];
  # No version constraint: the provider comes from nixpkgs, so flake.lock is the only pin.
  providers.hcloud.source = "hetznercloud/hcloud";

  # Resources shared by all Hetzner hosts of a herd.
  shared =
    { installPublicKey, herdName, ... }:
    {
      resource.hcloud_ssh_key.senn_install = {
        name = "senn-install-${herdName}";
        public_key = installPublicKey;
      };
    };

  tofu =
    { name, ... }:
    {
      config.resource.hcloud_server.${name} = {
        inherit name image location;
        server_type = type;
        ssh_keys = [ "\${hcloud_ssh_key.senn_install.id}" ];
        lifecycle = {
          # An adopted server may carry legacy pricing: never destroy it by plan.
          prevent_destroy = adopt != null;
          # Changing these replaces the server; after install they no longer matter.
          ignore_changes = [
            "ssh_keys"
            "image"
            "user_data"
          ];
        };
      };
      imports =
        if adopt == null then
          [ ]
        else
          [
            {
              to = "hcloud_server.${name}";
              id = toString adopt;
            }
          ];
      ipv4 = "\${hcloud_server.${name}.ipv4_address}";
      ipv6 = "\${hcloud_server.${name}.ipv6_address}";
    };

  # Hardware side: Hetzner Cloud VM, legacy BIOS boot, one disk.
  nixos =
    { lib, ... }:
    {
      imports = [
        inputs.srvos.nixosModules.hardware-hetzner-cloud
        inputs.disko.nixosModules.disko
      ];
      # GRUB for both firmware types (see the partitions below). As the removable EFI path
      # (\EFI\BOOT\BOOTX64.EFI) it needs no NVRAM entry, which a fresh VM doesn't have.
      boot.loader.grub = {
        efiSupport = true;
        efiInstallAsRemovable = true;
      };
      # cloud-init only sets up the network from Hetzner's metadata (IPv6 is static there); NixOS
      # owns everything else. Its default modules would manage users, passwords, SSH keys and the
      # hostname, and run the server's user_data (kept from its creation, and run again on a
      # rebuild): on NixOS they fail or undo what the host definition says. The network config
      # is applied in cloud-init's local stage, outside these module lists.
      services.cloud-init.settings = {
        datasource_list = [ "Hetzner" ];
        preserve_hostname = true;
        allow_userdata = false;
        cloud_init_modules = lib.mkForce [ ];
        cloud_config_modules = lib.mkForce [ ];
        cloud_final_modules = lib.mkForce [ ];
      };
      disko.devices.disk.main = {
        type = "disk";
        device = "/dev/sda";
        content = {
          type = "gpt";
          # Hetzner Cloud boots some server types with BIOS and others with UEFI, so the disk
          # carries both: a BIOS boot partition for GRUB's i386-pc stage and an EFI system
          # partition for its x86_64-efi image.
          partitions = {
            bios = {
              size = "1M";
              type = "EF02";
            };
            esp = {
              size = "512M";
              type = "EF00";
              content = {
                type = "filesystem";
                format = "vfat";
                mountpoint = "/boot";
                mountOptions = [ "umask=0077" ];
              };
            };
            root = {
              size = "100%";
              content = {
                type = "filesystem";
                format = "ext4";
                mountpoint = "/";
              };
            };
          };
        };
      };
    };
}
