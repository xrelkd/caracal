{ name
, version
, lib
, rustPlatform
, protobuf
, trunk
, binaryen
, wasm-bindgen-cli
, installShellFiles
}:

rustPlatform.buildRustPackage {
  pname = name;
  inherit version;

  src = lib.cleanSource ./..;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  nativeBuildInputs = [
    protobuf

    binaryen
    trunk
    wasm-bindgen-cli

    installShellFiles
  ];

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
