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
      "accesskit-0.16.0" = "sha256-uoLcd116WXQTu1ZTfJDEl9+3UPpGBN/QuJpkkGyRADQ=";
      "clipboard_macos-0.1.0" = "sha256-+8CGmBf1Gl9gnBDtuKtkzUE5rySebhH7Bsq/kNlJofY=";
      "cosmic-client-toolkit-0.1.0" = "sha256-KvXQJ/EIRyrlmi80WKl2T9Bn+j7GCfQlcjgcEVUxPkc=";
      "cosmic-text-0.15.0" = "sha256-+kptqjcP0UL/aIHcTLlIziRYJ2UMutYD75T07YGraGY=";
      "dpi-0.1.1" = "sha256-zuX4cvJP67wR4SyWIfkqdxnEf+SUgBb0//1hpoZszRo=";
      "iced-0.14.0-dev" = "sha256-MuPLf99Msg2VK0XCdQlEs2Et8x8kBiuJiyP/JWAtwzQ=";
      "iced_glyphon-0.6.0" = "sha256-u1vnsOjP8npQ57NNSikotuHxpi4Mp/rV9038vAgCsfQ=";
      "smithay-clipboard-0.8.0" = "sha256-4InFXm0ahrqFrtNLeqIuE3yeOpxKZJZx+Bc0yQDtv34=";
      "softbuffer-0.4.1" = "sha256-/ocK79Lr5ywP/bb5mrcm7eTzeBbwpOazojvFUsAjMKM=";
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
    libxkbcommon

    vulkan-loader
  ];

  postFixup = ''
    patchelf $out/bin/iwwc \
      --add-rpath ${lib.makeLibraryPath (with pkgs; [vulkan-loader libxkbcommon wayland])}
  '';
}
