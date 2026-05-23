{
  lib,
  rustPlatform,
  pkg-config,
  pkgs,
}:
rustPlatform.buildRustPackage rec {
  pname = "iwwc";
  version = "${(builtins.fromTOML (builtins.readFile (src + "/Cargo.toml"))).package.version}-git";

  src = lib.cleanSource ../.;

  cargoLock = {
    lockFile = "${src}/Cargo.lock";
    allowBuiltinFetchGit = true;
  };

  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  buildInputs = with pkgs; [
    pango
    glib
    gdk-pixbuf
    atkmm
    libxkbcommon

    vulkan-loader
  ];

  postFixup = ''
    patchelf $out/bin/iwwc \
      --add-rpath ${lib.makeLibraryPath (with pkgs; [vulkan-loader libxkbcommon wayland])}
  '';
}
