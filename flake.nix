{
  description = "Iced Wayland Widget Center";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default-linux";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    systems,
    ...
  }: let
    eachSystem = nixpkgs.lib.genAttrs (import systems);

    pkgsFor = system:
      import nixpkgs {
        inherit system;
        overlays = [];
      };
  in {
    packages = eachSystem (system: rec {
      default = iwwc;
      iwwc = nixpkgs.legacyPackages.${system}.callPackage ./nix/package.nix {};
    });

    overlays.default = final: prev: {
      iwwc = self.packages.${prev.system}.default;
    };

    devShells = eachSystem (system: {
      default = (pkgsFor system).callPackage ./nix/shell.nix {};
    });

    nixosModules.default = import ./nix/module.nix {inherit self;};
    homeModules.default = import ./nix/module.nix {
      isHome = true;
      inherit self;
    };

    formatter.x86_64-linux = inputs.nixpkgs.legacyPackages.x86_64-linux.alejandra;
  };
}
