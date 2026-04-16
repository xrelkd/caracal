{
  name,
  version,
  lib,
  nfpm,
  caracal-static,
  pkgs,
  stdenv,
  packager ? "deb",
  arch ? "amd64",
}:

let
  nfpmConfig = pkgs.replaceVars ./nfpm.yaml {
    NAME = name;
    VERSION = version;
    ARCH = arch;
  };
in
stdenv.mkDerivation {
  pname = "${name}-${packager}";
  inherit version;

  nativeBuildInputs = [ nfpm ];

  dontUnpack = true;
  dontConfigure = true;
  dontBuild = true;

  installPhase = ''
    runHook preInstall

    staging=$(mktemp -d)
    mkdir -p "$staging/usr/bin"
    mkdir -p "$staging/usr/share/bash-completion/completions"
    mkdir -p "$staging/usr/share/fish/vendor_completions.d"
    mkdir -p "$staging/usr/share/zsh/site-functions"

    cp ${caracal-static}/bin/caracal "$staging/usr/bin/"
    cp ${caracal-static}/bin/caracal-daemon "$staging/usr/bin/"
    cp ${caracal-static}/bin/caracal-tui "$staging/usr/bin/"

    cp ${caracal-static}/share/bash-completion/completions/* "$staging/usr/share/bash-completion/completions/"
    cp ${caracal-static}/share/fish/vendor_completions.d/* "$staging/usr/share/fish/vendor_completions.d/"
    cp ${caracal-static}/share/zsh/site-functions/* "$staging/usr/share/zsh/site-functions/"

    mkdir -p $out
    cd "$staging"
    nfpm package -f ${nfpmConfig} --packager ${packager} --target "$out"

    runHook postInstall
  '';

  meta = with lib; {
    description = "File downloader written in Rust Programming Language (statically linked, ${packager} package)";
    homepage = "https://github.com/xrelkd/caracal";
    license = licenses.gpl3Only;
    platforms = platforms.linux;
    maintainers = with maintainers; [ xrelkd ];
  };
}
