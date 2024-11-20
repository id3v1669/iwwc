{ lib
, rustPlatform
, makeWrapper
, pkg-config
, pkgs
}:
rustPlatform.buildRustPackage rec {

  pname = "rs-nc";
  version = "0.0.1";

  src = lib.cleanSource ../.;

  cargoLock = {
    lockFile = "${src}/Cargo.lock";
    outputHashes = {
      "dpi-0.1.1" = "sha256-25sOvEBhlIaekTeWvy3UhjPI1xrJbOQvw/OkTg12kQY=";
      "glyphon-0.5.0" = "sha256-VgeV/5nYQwzYQWN8/e/kCljqv3vB7McJb4aA6OhhrOI=";
      "iced-0.14.0-dev" = "sha256-d4j5GT2PPLmsRrBuWZd4L/4ApWA9RiAR1Kdr+KzImXY=";
      "iced_layershell-0.9.7" = "sha256-o1/SHx2yW6znbf6BA8i5jQL4Dq+JD1hxqILNXSjIz5k=";
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
    patchelf $out/bin/rs-nc \
      --add-rpath ${lib.makeLibraryPath (with pkgs; [ vulkan-loader xorg.libX11 libxkbcommon wayland ])}
  '';
}