{
  rustToolchain,
  cargoArgs,
  unitTestArgs,
  pkgs,
  lib,
  stdenv,
  darwin,
  ...
}:

let
  cargo-ext = pkgs.callPackage ./cargo-ext.nix { inherit cargoArgs unitTestArgs; };
in
pkgs.mkShell {
  name = "dev-shell";

  buildInputs = lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.SystemConfiguration
  ];

  nativeBuildInputs = with pkgs; [
    cargo-ext.cargo-build-all
    cargo-ext.cargo-clippy-all
    cargo-ext.cargo-doc-all
    cargo-ext.cargo-nextest-all
    cargo-ext.cargo-test-all
    cargo-nextest
    rustToolchain

    nodejs
    pnpm
    typescript

    tokei

    protobuf

    jq

    # clang-tools contains clang-format
    clang-tools
    biome
    hclfmt
    nixfmt
    prettier
    shfmt
    taplo
    treefmt

    shellcheck
    typos

    libgit2
    pkg-config
  ];

  shellHook = ''
    export NIX_PATH="nixpkgs=${pkgs.path}"

    # This allows the compiled build-script-build to find libgit2 at runtime
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [ pkgs.libgit2 ]}:$LD_LIBRARY_PATH"
  '';
}
