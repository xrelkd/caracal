{
  name,
  version,
  ocelot-static,
  target ? "x86_64-unknown-linux-musl",
  runCommand,
}:

runCommand "${name}-${version}-${target}.tar.gz" { } ''
  mkdir -p $out
  tar czvf $out/${name}-${version}-${target}.tar.gz \
    -C ${ocelot-static} \
    bin \
    share/bash-completion \
    share/fish \
    share/zsh
''
