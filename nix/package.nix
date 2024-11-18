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

  cargoLock.lockFile = "${src}/Cargo.lock";

  nativeBuildInputs = with pkgs; [ pkg-config ];

  buildInputs = with pkgs; [
    pango
    glib
    gdk-pixbuf
    atkmm

    #libGL
    fontconfig
    vulkan-loader
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    #dbus
  ];

  # LD_LIBRARY_PATH = "${lib.makeLibraryPath (with pkgs; [ libxkbcommon wayland libGL ])}";

  # wrap the binary to set the LD_LIBRARY_PATH
  # dontPatchELF = true;

  # postInstall = ''
  #   patchelf --set-rpath ${lib.makeLibraryPath runLibs}:$(patchelf --print-rpath $out/bin/rs-nc) $out/bin/rs-nc
  # '';
  postFixup = ''
    patchelf $out/bin/rs-nc \
      --add-rpath ${lib.makeLibraryPath (with pkgs; [ vulkan-loader xorg.libX11 libxkbcommon wayland ])}
  '';
}