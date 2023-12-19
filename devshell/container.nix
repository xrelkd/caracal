{ name
, version
, dockerTools
, caracal
, buildEnv
, ...
}:

dockerTools.buildImage {
  inherit name;
  tag = "v${version}";

  copyToRoot = buildEnv {
    name = "image-root";
    paths = [ caracal ];
    pathsToLink = [ "/bin" ];
  };

  config = {
    Entrypoint = [ "${caracal}/bin/caracal-daemon" ];
  };
}
