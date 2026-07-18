{
  self,
  isHome ? false,
}: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.programs.iwwc;
  inherit (pkgs.stdenv.hostPlatform) system;
  inherit (lib) types;
  inherit (lib.modules) mkIf;
  inherit (lib.options) mkOption mkEnableOption;
in {
  options.programs.iwwc = {
    enable = mkEnableOption "Iced Wayland Widget Center";

    package = mkOption {
      description = "The package to use for `iwwc`";
      default = self.packages.${system}.default;
      type = types.package;
    };
  };

  config = mkIf cfg.enable (
    lib.optionalAttrs isHome {
      home.packages = [cfg.package];

      systemd.user.services.iwwc = {
        Unit = {
          Description = "Iced Wayland Widget Center Daemon";
          After = ["graphical-session.target"];
          PartOf = ["graphical-session.target"];
        };
        Service = {
          ExecStart = "${cfg.package}/bin/iwwc daemon";
          Restart = "always";
          Environment = [
            "PATH=${lib.makeBinPath [
              "/run/wrappers"
              "/etc/profiles/per-user/%u"
              "/run/current-system/sw"
            ]}"
          ];
        };
        Install = {WantedBy = ["graphical-session.target"];};
      };
    }
    // lib.optionalAttrs (!isHome) {
      environment.systemPackages = [cfg.package];

      systemd.user.services.iwwc = {
        description = "Iced Wayland Widget Center Daemon";
        bindsTo = ["graphical-session.target"];
        after = ["graphical-session.target"];
        script = "${cfg.package}/bin/iwwc daemon";
        serviceConfig.Restart = "always";
        path = [
          "/run/wrappers"
          "/etc/profiles/per-user/%u"
          "/run/current-system/sw"
        ];
        wantedBy = ["graphical-session.target"];
      };
    }
  );
}
