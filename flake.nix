{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";

    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs =
    { self, nixpkgs, ... }@inputs:
    inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import inputs.rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        toml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        name = toml.package.name;
        version = toml.package.version;

        formattingConfig =
          { ... }:
          {
            projectRootFile = "flake.nix";
            programs = {
              nixfmt.enable = true;
              rustfmt = {
                enable = true;
                edition = toml.package.edition;
              };
              sql-formatter = {
                enable = true;
                dialect = "sqlite";
              };

              mdformat.enable = true;
              jsonfmt.enable = true;

              # js / ts / css / scss
              prettier.enable = true;

              leptosfmt.enable = true;

              toml-sort.enable = true;

            };
          };

        treeFmtEval = inputs.treefmt-nix.lib.evalModule pkgs formattingConfig;
        crane_pkg = pkgs.callPackage ./pkg_crane.nix {
          inherit
            name
            version
            inputs
            toml
            ;
        };
      in
      {
        nixosModules = rec {
          default = import ./module.nix self system;
          strichliste = default;
        };
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            (rust-bin.stable.latest.default.override {
              targets = [ "wasm32-unknown-unknown" ];
            })
            # openssl
            # pkg-config
            rust-analyzer
            cargo-leptos # does work. Make sure '/home/ole/.cargo/bin/cargo-leptos' does not exist
            lld
            dart-sass
            sqlite.dev
            sqlx-cli
            rustfmt
            stylance-cli
            clippy
            vscode-langservers-extracted
            binaryen
            cargo-generate
            tailwindcss
            tailwindcss-language-server
            vscode-extensions.bradlc.vscode-tailwindcss
          ];

          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          DATABASE_URL = "sqlite:tmp/db.sqlite";
        };

        formatter = treeFmtEval.config.build.wrapper;

        inherit (crane_pkg) packages;

        checks = crane_pkg.checks // {
          formatting = treeFmtEval.config.build.check self;
          directoryStructureReadMe = pkgs.stdenv.mkDerivation {
            name = "directoryStructureReadMe";
            src = ./.;
            nativeBuildInputs = with pkgs; [
              python3
              gawk
              coreutils
            ];
            buildPhase = ''
              patchShebangs .
              ./scripts/diff-file-structure.sh
              touch "$out"
            '';
          };
        };
      }
    );
}
