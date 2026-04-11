{
  name,
  version,
  lib,
  rustPlatform,
  protobuf,
  installShellFiles,
  completions ? null,
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
    installShellFiles
  ];

  doCheck = false;

  postInstall =
    if completions != null then
      ''
        mkdir -p $out/share
        cp -r ${completions}/share/* $out/share/
      ''
    else
      ''
        for cmd in caracal caracal-daemon caracal-tui; do
          installShellCompletion --cmd $cmd \
            --bash <($out/bin/$cmd completions bash) \
            --fish <($out/bin/$cmd completions fish) \
            --zsh  <($out/bin/$cmd completions zsh)
        done
      '';

  meta = with lib; {
    description = "File downloader written in Rust Programming Language (statically linked)";
    homepage = "https://github.com/xrelkd/caracal";
    license = licenses.gpl3Only;
    platforms = platforms.linux;
    maintainers = with maintainers; [ xrelkd ];
    mainProgram = "caracal";
  };
}
