{
  lib,
  rustPlatform,
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
      "cryoglyph-0.1.0" = "sha256-Jc+rhzd5BIT7aYBtIfsBFFKkGChdEYhDHdYGiv4KE+c=";
      "dpi-0.1.1" = "sha256-hlVhlQ8MmIbNFNr6BM4edKdZbe+ixnPpKm819zauFLQ=";
      "iced-0.14.0-dev" = "sha256-1svvPtYyjL4/0ESRGSXPfWU6JoWqKMO147849vmD7hs=";
      "iced_exdevtools-0.14.0-dev" = "sha256-0Lp95CsLbM9byBxW8tP5UQuiJSUgA9QNYQtBRgi6JNI=";
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

    vulkan-loader
  ];

  postFixup = ''
    patchelf $out/bin/iwwc \
      --add-rpath ${lib.makeLibraryPath (with pkgs; [vulkan-loader libxkbcommon wayland])}
  '';
}
