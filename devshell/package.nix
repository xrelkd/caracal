{
  name,
  version,
  lib,
  stdenv,
  rustPlatform,
  protobuf,
  nodejs,
  pnpm,
  fetchPnpmDeps,
  pnpmConfigHook,
  installShellFiles,
  darwin,
}:
let
  web-ui-dist = stdenv.mkDerivation {
    inherit version;
    pname = "${name}-web-ui";
    src = lib.cleanSourceWith {
      src = ./../crates/web-ui/ui;
      filter =
        path: type: (lib.hasInfix "/out" path == false) && (lib.hasInfix "/node_modules" path == false);
    };

    nativeBuildInputs = [
      nodejs
      pnpm
      pnpmConfigHook
    ];

    pnpmDeps = fetchPnpmDeps {
      inherit (web-ui-dist) pname version src;
      fetcherVersion = 1;
      hash = "sha256-jfFbsm8z1TiwCieDjn9/hh/KeOXnl9wWkKxS5xpFu/k=";
    };

    buildPhase = ''
      runHook preBuild
      pnpm build
      runHook postBuild
    '';

    installPhase = ''
      runHook preInstall
      mkdir -p $out
      cp -r out/* $out/
      runHook postInstall
    '';
  };
in
rustPlatform.buildRustPackage {
  pname = name;
  inherit version;

  src = lib.cleanSource ./..;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  buildInputs = lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.SystemConfiguration
  ];

  nativeBuildInputs = [
    protobuf

    installShellFiles
  ];

  PREBUILT_WEBUI_DIST = "${web-ui-dist}";

  postInstall = ''
    for cmd in caracal caracal-daemon caracal-tui; do
      installShellCompletion --cmd $cmd \
        --bash <($out/bin/$cmd completions bash) \
        --fish <($out/bin/$cmd completions fish) \
        --zsh  <($out/bin/$cmd completions zsh)
    done
  '';

  meta = with lib; {
    description = "File downloader written in Rust Programming Language";
    homepage = "https://github.com/xrelkd/caracal";
    license = licenses.gpl3Only;
    maintainers = with maintainers; [ xrelkd ];
    mainProgram = "caracal";
  };
}
