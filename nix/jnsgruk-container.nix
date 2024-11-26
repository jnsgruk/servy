{
  buildEnv,
  cacert,
  dockerTools,
  jnsgruk,
  lib,
  version,
}:
dockerTools.buildImage {
  name = "jnsgruk/jnsgr.uk";
  tag = version;
  created = "now";
  copyToRoot = buildEnv {
    name = "image-root";
    paths = [
      jnsgruk
      cacert
    ];
    pathsToLink = [
      "/bin"
      "/etc/ssl/certs"
    ];
  };
  config = {
    Entrypoint = [ "${lib.getExe jnsgruk}" ];
    Expose = [
      8080
      8801
    ];
    User = "10000:10000";
  };
}
