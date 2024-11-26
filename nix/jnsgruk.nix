{
  hugo,
  jnsgruk-content,
  lib,
  openssl,
  pkg-config,
  rustPlatform,
  version,
}:
rustPlatform.buildRustPackage {
  pname = "jnsgruk";
  version = version;

  src = lib.cleanSource ../.;

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
    hugo
  ];

  buildInputs = [
    openssl
  ];

  cargoFlags = [
    "--bin"
    "jnsgruk"
  ];

  env.SERVY_ASSETS_DIR = "${jnsgruk-content}/share/jnsgruk/public";

  doCheck = false;

  meta = {
    mainProgram = "jnsgruk";
    description = "Self-contained personal website for jnsgruk";
    homepage = "https://github.com/jnsgruk/jnsgr.uk";
    license = lib.licenses.asl20;
    platforms = lib.platforms.all;
    maintainers = with lib.maintainers; [ jnsgruk ];
  };
}
