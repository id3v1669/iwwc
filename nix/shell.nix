{pkgs}:
pkgs.mkShell {
  name = "rs-nc devShell";
  nativeBuildInputs = with pkgs; [
    # Compilers
    cargo
    rustc
    scdoc

    # Tools
    cargo-audit
    cargo-deny
    clippy
    rust-analyzer
    rustfmt
  ];
}