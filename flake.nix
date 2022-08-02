{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    naersk,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages."${system}";
        naersk-lib = naersk.lib."${system}";
      in rec {
        # `nix build`
        packages.prometheus-mdns-sd = naersk-lib.buildPackage {
          pname = "prometheus-mdns-sd";
          root = ./.;
        };
        defaultPackage = packages.prometheus-mdns-sd;
        defaultApp = packages.prometheus-mdns-sd;

        # `nix develop`
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [rustc cargo bacon cargo-edit cargo-outdated];
        };
      }
    )
    // {
      nixosModule = {
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
          };

          config = mkIf cfg.enable {
            systemd.services."prometheus-mdns-sd" = let
              pkg = self.defaultPackage.${pkgs.system};
            in {
              wantedBy = ["multi-user.target"];

              serviceConfig = {
                ExecStart = "${pkg}/bin/prometheus-mdns-sd-rs ${cfg.target}";
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
        };
    };
}
