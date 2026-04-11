{
  description = "Caracal - File downloader written in Rust Programming Language";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-parts,
      fenix,
      crane,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {

      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      flake = {
        overlays.default = final: prev: { };
      };

      perSystem =
        {
          config,
          self',
          inputs',
          pkgs,
          system,
          ...
        }:
        let

          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              self.overlays.default
              fenix.overlays.default
            ];
          };

          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          name = "caracal";
          version = cargoToml.workspace.package.version;

          rustToolchain =
            with fenix.packages.${system};
            combine [
              stable.rustc
              stable.cargo
              stable.clippy
              stable.rust-src
              stable.rust-std
              targets.x86_64-unknown-linux-musl.stable.rust-std
              targets.aarch64-unknown-linux-musl.stable.rust-std
              default.rustfmt
            ];

          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };

          rustPlatformMusl = pkgs.pkgsStatic.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };

          isCross = system == "x86_64-linux";
          isCrossFromAarch64 = system == "aarch64-linux";

          crossPkgs =
            if isCross then
              import nixpkgs {
                inherit system;
                crossSystem = {
                  config = "aarch64-unknown-linux-musl";
                };
                overlays = [
                  self.overlays.default
                  fenix.overlays.default
                ];
              }
            else if isCrossFromAarch64 then
              import nixpkgs {
                inherit system;
                crossSystem = {
                  config = "x86_64-unknown-linux-musl";
                };
                overlays = [
                  self.overlays.default
                  fenix.overlays.default
                ];
              }
            else
              null;

          rustPlatformCrossMusl =
            if isCross || isCrossFromAarch64 then
              crossPkgs.pkgsStatic.makeRustPlatform {
                cargo = rustToolchain;
                rustc = rustToolchain;
              }
            else
              null;

          cargoArgs = [
            "--workspace"
            "--bins"
            "--examples"
            "--tests"
            "--benches"
            "--all-targets"
          ];
          unitTestArgs = [ "--workspace" ];
        in
        {

          formatter = pkgs.treefmt;

          devShells.default = pkgs.callPackage ./devshell {
            inherit
              rustToolchain
              cargoArgs
              unitTestArgs
              ;
          };

          packages = rec {
            default = caracal;
            caracal = pkgs.callPackage ./devshell/package.nix {
              inherit name version rustPlatform;
              inherit (pkgs) darwin;
            };
            caracal-static = pkgs.pkgsStatic.callPackage ./devshell/package-static.nix {
              inherit name version;
              rustPlatform = rustPlatformMusl;
            };
            completions = pkgs.runCommand "caracal-completions" { } ''
              mkdir -p $out/share/{bash-completion/completions,fish/vendor_completions.d,zsh/site-functions}
              ${caracal}/bin/caracal completions bash  > $out/share/bash-completion/completions/caracal
              ${caracal}/bin/caracal completions fish  > $out/share/fish/vendor_completions.d/caracal.fish
              ${caracal}/bin/caracal completions zsh   > $out/share/zsh/site-functions/_caracal
              ${caracal}/bin/caracal-daemon completions bash  > $out/share/bash-completion/completions/caracal-daemon
              ${caracal}/bin/caracal-daemon completions fish  > $out/share/fish/vendor_completions.d/caracal-daemon.fish
              ${caracal}/bin/caracal-daemon completions zsh   > $out/share/zsh/site-functions/_caracal-daemon
              ${caracal}/bin/caracal-tui completions bash  > $out/share/bash-completion/completions/caracal-tui
              ${caracal}/bin/caracal-tui completions fish  > $out/share/fish/vendor_completions.d/caracal-tui.fish
              ${caracal}/bin/caracal-tui completions zsh   > $out/share/zsh/site-functions/_caracal-tui
            '';
            container = pkgs.callPackage ./devshell/container.nix {
              inherit name version caracal;
              inherit (pkgs) darwin;
            };
            check-format = pkgs.callPackage ./devshell/format.nix { };
            deb-x86_64 = pkgs.callPackage ./devshell/package-nfpm.nix {
              inherit name version;
              ocelot-static = if isCrossFromAarch64 then static-x86_64 else caracal-static;
              packager = "deb";
              arch = "amd64";
            };
            rpm-x86_64 = pkgs.callPackage ./devshell/package-nfpm.nix {
              inherit name version;
              ocelot-static = if isCrossFromAarch64 then static-x86_64 else caracal-static;
              packager = "rpm";
              arch = "x86_64";
            };
            apk-x86_64 = pkgs.callPackage ./devshell/package-nfpm.nix {
              inherit name version;
              ocelot-static = if isCrossFromAarch64 then static-x86_64 else caracal-static;
              packager = "apk";
              arch = "x86_64";
            };
            tarball-x86_64 = pkgs.callPackage ./devshell/package-tarball.nix {
              inherit name version;
              ocelot-static = if isCrossFromAarch64 then static-x86_64 else caracal-static;
            };
            static-aarch64 =
              if isCross then
                crossPkgs.pkgsStatic.callPackage ./devshell/package-static.nix {
                  inherit name version completions;
                  rustPlatform = rustPlatformCrossMusl;
                }
              else
                caracal-static;
            static-x86_64 =
              if isCrossFromAarch64 then
                crossPkgs.pkgsStatic.callPackage ./devshell/package-static.nix {
                  inherit name version completions;
                  rustPlatform = rustPlatformCrossMusl;
                }
              else
                caracal-static;
            deb-aarch64 = pkgs.callPackage ./devshell/package-nfpm.nix {
              inherit name version;
              ocelot-static = static-aarch64;
              packager = "deb";
              arch = "arm64";
            };
            rpm-aarch64 = pkgs.callPackage ./devshell/package-nfpm.nix {
              inherit name version;
              ocelot-static = static-aarch64;
              packager = "rpm";
              arch = "aarch64";
            };
            apk-aarch64 = pkgs.callPackage ./devshell/package-nfpm.nix {
              inherit name version;
              ocelot-static = static-aarch64;
              packager = "apk";
              arch = "aarch64";
            };
            tarball-aarch64 = pkgs.callPackage ./devshell/package-tarball.nix {
              inherit name version;
              ocelot-static = static-aarch64;
              target = "aarch64-unknown-linux-musl";
            };
          };
        };
    };
}
