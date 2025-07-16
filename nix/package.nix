{
  lib,
  rustPlatform,
  makeWrapper,
  pkg-config,
  pkgs,
}:
rustPlatform.buildRustPackage rec {
  pname = "iwwc";
  version = "0.1.0";

  src = lib.cleanSource ../.;

  cargoLock = {
    lockFile = "${src}/Cargo.lock";
    outputHashes = {
      "cryoglyph-0.1.0" = "sha256-X7S9jq8wU6g1DDNEzOtP3lKWugDnpopPDBK49iWvD4o=";
      "dpi-0.1.1" = "sha256-hlVhlQ8MmIbNFNr6BM4edKdZbe+ixnPpKm819zauFLQ=";
      "iced-0.14.0-dev" = "sha256-gPz/J9+agGiyA9DIdFkcR7nvBeKcalXaHeHLwGZJ77I=";
      "iced_exdevtools-0.14.0-dev" = "sha256-ViPH4HPI+NdQ9+DHnavfyrdaMWuhoDWeBfoKfGGn4d0=";
    };
  };

  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  buildInputs = with pkgs; [
    pango
    glib
    gdk-pixbuf
    atkmm

    fontconfig
    vulkan-loader
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
  ];

  postFixup = ''
    patchelf $out/bin/iwwc \
      --add-rpath ${lib.makeLibraryPath (with pkgs; [vulkan-loader xorg.libX11 libxkbcommon wayland])}
  '';
}
