{
  lib,
  rustPlatform,
  pkg-config,
  openssl,
  stdenv,
}: let
  cargoToml = lib.importTOML ../Cargo.toml;
in
  rustPlatform.buildRustPackage {
    pname = cargoToml.package.name;
    version = cargoToml.package.version;

    src = lib.cleanSource ../.;

    cargoLock = {
      lockFile = ../Cargo.lock;
    };

    doCheck = false;

    nativeBuildInputs = [pkg-config];

    buildInputs = lib.optionals stdenv.hostPlatform.isLinux [ openssl ];

    env.OPENSSL_NO_VENDOR = 1;

    meta = {
      description = "Subsonic API-compatible music streaming server in Rust";
      homepage = "https://github.com/imaviso/suboxide";
      license = lib.licenses.mit;
      mainProgram = "suboxide";
      platforms = lib.platforms.unix;
    };
  }
