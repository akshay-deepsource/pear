{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-21.05";
    naersk.url = "github:nmattia/naersk";
    mozillapkgs = {
      url = "github:mozilla/nixpkgs-mozilla/";
      flake = false;
    };
    gitignore = {
      url = "github:hercules-ci/gitignore";
      flake = false;
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, naersk, mozillapkgs, gitignore, ... }:
    let

      supportedSystems = [ "x86_64-darwin" ];
      forAllSystems = f: nixpkgs.lib.genAttrs supportedSystems (system: f system);

      nixpkgsFor = forAllSystems (system:
        import nixpkgs {
          inherit system;
          overlays = [ self.overlay ];
        }
      );

      pkgs = nixpkgsFor.${"x86_64-darwin"};

      lib = pkgs.lib;
      stdenv = pkgs.stdenv;
      darwin = pkgs.darwin;

      inherit (import gitignore { inherit (pkgs) lib; }) gitignoreSource;

      # Get a specific rust version
      mozilla = pkgs.callPackage (mozillapkgs + "/package-set.nix") { };
      chanspec = {
        date = "2021-10-26";
        channel = "nightly";
        sha256 = "1hLbypXA+nuH7o3AHCokzSBZAvQxvef4x9+XxO3aBao="; # set zeros after modifying channel or date
      };

      rustChannel = mozilla.rustChannelOf chanspec;
      rust = rustChannel.rust;
      rust-src = rustChannel.rust-src;

      naersk-lib = naersk.lib."${"x86_64-darwin"}".override {
        cargo = rust;
        rustc = rust;
      };

      nativeBuildInputs = with pkgs; [ ];

      buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
        darwin.apple_sdk.frameworks.Security
        darwin.apple_sdk.frameworks.CoreServices
        darwin.libiconv
        pkgs.nixUnstable
      ];

      # analyzer env vars
      buildVars = {
        CODE_PATH = "/code";
        ANALYSIS_CONFIG_PATH = "/toolbox/analyzer_config.json";
        ANALYSIS_RESULT_PATH = "/toolbox/analysis_results.json";
        AUTOFIX_CONFIG_PATH = "/toolbox/autofix_config.json";
        AUTOFIX_RESULT_PATH = "/toolbox/autofix_results.json";
        MARVIN_PATH = "/toolbox/marvin";
        RUST_SRC_PATH = "${rust-src}/lib/rustlib/src/rust/library";
        RUST_LOG = "info";
        RUST_BACKTRACE = 1;
      };

    in
    rec {

      overlay = final: prev: {

        muff = naersk-lib.buildPackage ({
          pname = "muff";
          version = "0.1.0";
          root = gitignoreSource ./.;
          cargoBuildOptions = x: x ++ [ "-p" "muff" ];
          inherit nativeBuildInputs buildInputs;
        } // buildVars);

        marvin-rust = naersk-lib.buildPackage ({
          pname = "analyzer";
          version = "0.1.0";
          root = gitignoreSource ./.;
          cargoBuildOptions = x: x ++ [ "-p" "analyzer" ];
          inherit nativeBuildInputs buildInputs;
        } // buildVars);

      };

      packages.marvin-rust = forAllSystems (system:
        (import nixpkgs {
          inherit system;
          overlays = [ self.overlay ];
        }).marvin-rust
      );

      defaultPackage = packages.marvin-rust;

      apps = forAllSystems
        (system:
          let
            muff =
              (import nixpkgs {
                inherit system;
                overlays = [ self.overlay ];
              }).muff;
          in
          {
            muff = {
              type = "app";
              program = "${muff}/bin/muff";
            };
          }
        );


      devShell = forAllSystems (system:
        pkgs.mkShell ({
          nativeBuildInputs = nativeBuildInputs ++ [
            rust
            rust-src
            pkgs.rust-analyzer
            pkgs.rustfmt
            pkgs.cargo
            pkgs.bacon

            pkgs.parallel
          ];
          inherit buildInputs;
        } // buildVars)
      );

    };
}
