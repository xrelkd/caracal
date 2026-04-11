{ pkgs }:

pkgs.runCommand "check-format"
  {
    buildInputs = with pkgs; [
      fd

      shellcheck

      clang-tools
      biome
      prettier
      nixfmt
      hclfmt
      shfmt
      taplo
      treefmt
    ];
  }
  ''
    # Copy source to a writable location
    export HOME=$TMPDIR
    cp -r ${./..} /tmp/check-format-src
    chmod -R +w /tmp/check-format-src

    cd /tmp/check-format-src

    treefmt \
      --allow-missing-formatter \
      --fail-on-change \
      --no-cache \
      --formatters clang-format \
      --formatters biome \
      --formatters prettier \
      --formatters nix \
      --formatters shell \
      --formatters hcl \
      --formatters toml

    # it worked!
    touch $out
  ''
