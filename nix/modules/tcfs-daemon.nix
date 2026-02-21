{ config, lib, pkgs, ... }:
# NixOS module: services.tcfsd
# Manages tcfsd as a systemd service with full hardening
#
# Example configuration:
#   services.tcfsd = {
#     enable = true;
#     configFile = "/etc/tcfs/config.toml";
#     credentialsFile = "/etc/tcfs/age-identity.txt";
#     mounts = [
#       { remote = "seaweedfs://dees-appu-bearts:8333/bucket"; local = "/mnt/tcfs"; }
#     ];
#   };

let
  cfg = config.services.tcfsd;
in {
  options.services.tcfsd = {
    enable = lib.mkEnableOption "tcfsd TummyCrypt filesystem daemon";

    package = lib.mkOption {
      type = lib.types.package;
      description = "tcfsd package to use";
    };

    configFile = lib.mkOption {
      type = lib.types.path;
      default = "/etc/tcfs/config.toml";
      description = "Path to tcfs.toml configuration file";
    };

    credentialsFile = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = "Path to age identity file for SOPS decryption";
    };

    mounts = lib.mkOption {
      type = lib.types.listOf (lib.types.submodule {
        options = {
          remote = lib.mkOption { type = lib.types.str; };
          local = lib.mkOption { type = lib.types.str; };
          readOnly = lib.mkOption { type = lib.types.bool; default = false; };
        };
      });
      default = [];
      description = "List of mounts to configure at startup";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.tcfsd = {
      description = "TummyCrypt filesystem daemon (tcfsd)";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "notify";
        ExecStart = "${cfg.package}/bin/tcfsd --config ${cfg.configFile}";
        Restart = "on-failure";
        RestartSec = "5s";

        # Security hardening
        DynamicUser = true;
        PrivateTmp = true;
        PrivateDevices = false; # FUSE needs /dev/fuse
        DeviceAllow = [ "/dev/fuse rw" ];
        ProtectSystem = "strict";
        ProtectHome = false; # May need to write to user mounts
        ReadWritePaths = cfg.mounts;
        NoNewPrivileges = true;
        CapabilityBoundingSet = [ "CAP_SYS_ADMIN" ]; # FUSE mount
        AmbientCapabilities = [ "CAP_SYS_ADMIN" ];

        # Credentials
        LoadCredentialEncrypted = lib.mkIf (cfg.credentialsFile != null)
          "age-identity:${cfg.credentialsFile}";
      };

      environment = {
        TCFS_CONFIG = cfg.configFile;
      } // lib.optionalAttrs (cfg.credentialsFile != null) {
        CREDENTIALS_DIRECTORY = "%d";
      };
    };

    environment.systemPackages = [ cfg.package ];
  };
}
