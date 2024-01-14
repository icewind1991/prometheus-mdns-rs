{
  config,
  lib,
  pkgs,
  ...
}:
with lib; let
  cfg = config.services.prometheus-mdns-sd;
in {
  options.services.prometheus-mdns-sd = {
    enable = mkEnableOption "WiFi prometheus exporter";

    target = mkOption {
      type = types.str;
      default = "/run/prometheus-mdns-sd/services.json";
      description = "json file to write the discovered services to";
    };

    package = mkOption {
      type = types.package;
      defaultText = literalExpression "pkgs.prometheus-mdns-sd";
      description = "package to use";
    };
  };

  config = mkIf cfg.enable {
    systemd.services."prometheus-mdns-sd" = {
      wantedBy = ["multi-user.target"];

      serviceConfig = {
        ExecStart = "${cfg.package}/bin/prometheus-mdns-sd-rs ${cfg.target}";
        Restart = "on-failure";
        DynamicUser = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        NoNewPrivileges = true;
        PrivateDevices = true;
        ProtectClock = true;
        CapabilityBoundingSet = true;
        ProtectKernelLogs = true;
        ProtectControlGroups = true;
        SystemCallArchitectures = "native";
        ProtectKernelModules = true;
        RestrictNamespaces = true;
        MemoryDenyWriteExecute = true;
        ProtectHostname = true;
        LockPersonality = true;
        ProtectKernelTunables = true;
        RestrictAddressFamilies = "AF_INET";
        RestrictRealtime = true;
        ProtectProc = "invisible";
        SystemCallFilter = ["@system-service" "~@resources" "~@privileged"];
        IPAddressDeny = "any";
        IPAddressAllow = ["multicast" "192.168.0.0/16"];
        PrivateUsers = true;
        ProcSubset = "pid";
        RuntimeDirectory = "prometheus-mdns-sd";
        RestrictSUIDSGID = true;
      };
    };
  };
}
