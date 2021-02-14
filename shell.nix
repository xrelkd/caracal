{ pkgs ? import ./nix { } }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [ git rustup ];

  buildInputs = with pkgs; [ xorg.libxcb ];

  RUST_BACKTRACE = "full";
}
