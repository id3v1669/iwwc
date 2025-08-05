{pkgs}:
pkgs.mkShell {
  name = "iwwc devShell";
  nativeBuildInputs = with pkgs; [
    # Compilers
    cargo
    rustc
    scdoc

    # Build Deps
    pkg-config
    pango
    glib
    gdk-pixbuf
    atkmm
    libxkbcommon

    # graphics
    vulkan-loader
    mesa # Mesa drivers, otherwise vulkan backend fails to see mesa from system

    # Wayland
    wayland
    wayland-protocols
    wayland-scanner

    # System
    dbus

    # Tools
    cargo-audit
    cargo-deny
    clippy
    rust-analyzer
    rustfmt
  ];

  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath (with pkgs; [
    vulkan-loader
    mesa
    libxkbcommon
    wayland
    wayland-protocols
  ])}";
}
