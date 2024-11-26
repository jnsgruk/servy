{
  lib,
  openssl,
  pkg-config,
  rustPlatform,
  version,
}:
rustPlatform.buildRustPackage {
  pname = "servy";
  version = version;

  src = lib.cleanSource ../.;

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    openssl
  ];

  cargoFlags = [
    "--bin"
    "servy"
  ];

  env.SERVY_ASSETS_DIR = "./tests/servy_assets";

  meta = {
    description = "A simple HTTP file server with some basic URL shortening/redirect functionality";
    homepage = "https://github.com/jnsgruk/servy";
    license = lib.licenses.asl20;
    mainProgram = "servy";
    platforms = lib.platforms.unix;
    maintainers = with lib.maintainers; [ jnsgruk ];
  };
}
