{
  lib,
  pkgs,
  toml,
  inputs,
  ...
}:
let
  src = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions [
      ./Cargo.toml
      ./Cargo.lock
      ./src
      ./public
      ./migrations
      ./.sqlx
      ./style
    ];
  };

  name = toml.package.name;
  version = toml.package.version;

  rustTarget = pkgs.rust-bin.stable.latest.minimal.override {
    extensions = [
      "rust-src"
      "clippy"
    ];
    targets = [ "wasm32-unknown-unknown" ];
  };

  craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustTarget;
  commonArgs = rec {
    inherit src version;
    strictDeps = true;
    pname = name;
    buildInputs = with pkgs; [
      cargo-leptos
      lld
      binaryen
      dart-sass
      sqlx-cli
      makeWrapper
      tailwindcss
    ];

    nativeBuildInputs = buildInputs;
  };

  frontendArtifacts = craneLib.buildDepsOnly (
    commonArgs
    // {
      CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
      pname = "${name}-frontend";
      doCheck = false;
    }
  );

  frontend = craneLib.buildPackage (
    commonArgs
    // {
      cargoArtifacts = frontendArtifacts;
      pname = "${name}-frontend";

      doNotPostBuildInstallCargoBinaries = true;
      buildPhaseCargoCommand = ''
        cargo leptos build --release -vvv --frontend-only
      '';

      installPhaseCommand = ''
        mkdir -p $out/site
        cp -r target/site/* $out/site
      '';
    }
  );

  serverArtifacts = craneLib.buildDepsOnly (
    commonArgs
    // {
      pname = "${name}-server";
      doCheck = false;
    }
  );

  server = craneLib.buildPackage (
    commonArgs
    // {
      pname = "${name}-server";
      cargoArtifacts = serverArtifacts;

      doNotPostBuildInstallCargoBinaries = true;
      buildPhaseCargoCommand = ''
        cargo leptos build --release -vvv --server-only
      '';

      nativeBuildInputs = commonArgs.buildInputs;

      installPhaseCommand = ''
        mkdir -p $out/bin
        cp target/release/${name} $out/bin
      '';
    }
  );

  package = pkgs.stdenv.mkDerivation {
    inherit name version;

    src = server; # some what arbitrarily, but has to be set to something
    installPhase = ''
      mkdir -p $out/bin/site

      cp ${server}/bin/* $out/bin
      cp -r ${frontend}/site $out/bin/
    '';
  };
  cargoClippyExtraArgsCommon = "--all-targets -- --deny warnings";
  clippyFrontend = craneLib.cargoClippy (
    commonArgs
    // {
      cargoArtifacts = frontendArtifacts;
      cargoClippyExtraArgs = "-F ssr ${cargoClippyExtraArgsCommon}";
    }
  );
  clippyServer = craneLib.cargoClippy (
    commonArgs
    // {
      cargoArtifacts = serverArtifacts;
      cargoClippyExtraArgs = "-F hydrate ${cargoClippyExtraArgsCommon}";
    }
  );
in
{
  packages.default = package;
  checks = {
    inherit clippyFrontend clippyServer;
  };
}
