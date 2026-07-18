# Installation

## Nix flake

The repository is a Nix flake exposing the package as `packages.<system>.default` and an overlay as `overlays.default`:

```nix
{ inputs, pkgs, ... }:
{
  nixpkgs.overlays = [ inputs.iwwc.overlays.default ];

  environment.systemPackages = [ pkgs.iwwc ];
  # or in home-manager:
  # home.packages = [ pkgs.iwwc ];
}
```

## From Source

```sh
git clone https://github.com/id3v1669/iwwc
cd iwwc
cargo build --release
```
