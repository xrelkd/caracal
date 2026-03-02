{ pkgs }:

pkgs.runCommandNoCC "check-format"
  {
    buildInputs = with pkgs; [
      fd

      shellcheck

      clang-tools
      biome
      nixpkgs-fmt
      shfmt
      taplo
      treefmt
    ];
  }
  ''
    treefmt \
      --allow-missing-formatter \
      --fail-on-change \
      --no-cache \
      --formatters biome \
      --formatters clang-format \
      --formatters nix \
      --formatters shell \
      --formatters hcl \
      --formatters toml \
      -C ${./..}

    # it worked!
    touch $out
  ''
