{ pkgs ? import ./nix { } }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [ git rustup ];

  buildInputs = with pkgs; [ ];

  RUST_BACKTRACE = "full";
}
