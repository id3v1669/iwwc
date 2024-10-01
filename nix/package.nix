{ lib
, rustPlatform
, makeWrapper
, pkg-config
, pkgs
}:
let
  runLibs = (with pkgs; [
    wayland
    libxkbcommon
    libGL
    dbus
  ]);
in
rustPlatform.buildRustPackage rec {

  pname = "rs-nc";
  version = "0.0.1";

  src = lib.cleanSource ../.;

  cargoLock.lockFile = "${src}/Cargo.lock";

  nativeBuildInputs = with pkgs; [ 
    pkg-config
    #makeWrapper
    # libxkbcommon
    # wayland
    # libGL
  ];

  buildInputs = with pkgs; [] ++ runLibs;

  # LD_LIBRARY_PATH = "${lib.makeLibraryPath (with pkgs; [ libxkbcommon wayland libGL ])}";

  # wrap the binary to set the LD_LIBRARY_PATH
  dontPatchELF = true;

  postInstall = ''
    patchelf --set-rpath ${lib.makeLibraryPath runLibs}:$(patchelf --print-rpath $out/bin/rs-nc) $out/bin/rs-nc
  '';
}