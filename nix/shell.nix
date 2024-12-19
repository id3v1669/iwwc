{pkgs}:
pkgs.mkShell {
  name = "rs-nc devShell";
  nativeBuildInputs = with pkgs; [
    # Compilers
    cargo
    rustc
    scdoc

    # build Deps
    pkg-config
    pango
    glib
    gdk-pixbuf
    atkmm

    libxkbcommon

    # other
    wayland
    wayland-protocols
    dbus
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    

    # Tools
    cargo-audit
    cargo-deny
    clippy
    rust-analyzer
    rustfmt
  ];

  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath (with pkgs; [
    vulkan-loader
    xorg.libX11
    libxkbcommon
    wayland
    wayland-protocols
  ])}";
}