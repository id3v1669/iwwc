{pkgs}:
let
  runDeps = with pkgs; [
    wayland
    libGL
    libxkbcommon

    # needed for layershell single Application
    # for unknow reason works without it with MultiApplication
    vulkan-loader
  ];
in
pkgs.mkShell {
  name = "rs-nc devShell";
  nativeBuildInputs = with pkgs; [
    # Compilers
    cargo
    rustc
    scdoc

    # build Deps
    pkg-config
    gtk3
    librsvg
    gdb
    

    # Tools
    cargo-audit
    cargo-deny
    clippy
    rust-analyzer
    rustfmt
  ] ++ runDeps;

  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath runDeps}";
}