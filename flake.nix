{
  description = "A recreation of a favorite childhood flash game";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, crane, fenix, flake-utils, advisory-db, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        inherit (pkgs) lib;

        craneLib = crane.mkLib pkgs;
        src = craneLib.cleanCargoSource ./.;

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          inherit src;
          strictDeps = true;

          nativeBuildInputs = [
            pkgs.mold-wrapped
            pkgs.llvmPackages.clangUseLLVM
            pkgs.pkg-config
            pkgs.alsa-lib
          ];

          buildInputs = [
            pkgs.alsa-lib.dev
            # Add additional build inputs here
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];

          # Additional environment variables can be set directly
          # MY_CUSTOM_VAR = "some value";
        };

        craneLibLLvmTools = craneLib.overrideToolchain
          (fenix.packages.${system}.complete.withComponents [
            "cargo"
            "llvm-tools"
            "rustc"
          ]);

        # Build *just* the cargo dependencies (of the entire workspace),
        # so we can reuse all of that work (e.g. via cachix) when running in CI
        # It is *highly* recommended to use something like cargo-hakari to avoid
        # cache misses when building individual top-level-crates
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        individualCrateArgs = commonArgs // {
          inherit cargoArtifacts;
          inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
          # NB: we disable tests since we'll run them all via cargo-nextest
          doCheck = false;
        };

        workspaceCrates = {
           n_dit = ./crates/n_dit;
           charmi = ./crates/charmi;
           cq_term = ./crates/cq_term;
           game_core = ./crates/game_core;
           charmi_bevy = ./crates/charmi_bevy;
           charmi_macros = ./crates/charmi_macros;
        };
        # TODO figure out including assets in src

        fileSetForCrates = crateNames: lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions ([
            ./Cargo.toml
            ./Cargo.lock ]
            ++ (builtins.map (crateName: craneLib.fileset.commonCargoSources workspaceCrates.${crateName}) crateNames));
        };

        # Build the top-level crates of the workspace as individual derivations.
        # This allows consumers to only depend on (and build) only what they need.
        # Though it is possible to build the entire workspace as a single derivation,
        # so this is left up to you on how to organize things
        #
        # Note that the cargo workspace must define `workspace.members` using wildcards,
        # otherwise, omitting a crate (like we do below) will result in errors since
        # cargo won't be able to find the sources for all members.
        # n_dit = craneLib.buildPackage (individualCrateArgs // {
        #   pname = "n_dit";
        #   # cargoExtraArgs = "-p my-cli";
        #   src = fileSetForCrate ./.;
        # });
        charmi = craneLib.buildPackage (individualCrateArgs // {
          pname = "charmi";
          cargoExtraArgs = "-p charmi";
          src = fileSetForCrates ["charmi"];
        });
        game_core = craneLib.buildPackage (individualCrateArgs // {
          pname = "game_core";
          cargoExtraArgs = "-p game_core";
          src = fileSetForCrates ["game_core"];
        });
        cq_term = craneLib.buildPackage (individualCrateArgs // {
          pname = "cq_term";
          cargoExtraArgs = "-p cq_term";
          src = fileSetForCrates ["cq_term" "charmi" "game_core" "charmi_bevy" "charmi_macros"];
        });
        n_dit = craneLib.buildPackage (individualCrateArgs // {
          pname = "n_dit";
          cargoExtraArgs = "-p n_dit";
          src = fileSetForCrates ["n_dit" "cq_term" "charmi" "game_core" "charmi_bevy" "charmi_macros"];
        });
      in
      {
        checks = {
          # Build the crates as part of `nix flake check` for convenience
          inherit charmi;

          # Run clippy (and deny all warnings) on the workspace source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          my-workspace-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          my-workspace-doc = craneLib.cargoDoc (commonArgs // {
            inherit cargoArtifacts;
          });

          # Check formatting
          my-workspace-fmt = craneLib.cargoFmt {
            inherit src;
          };

          my-workspace-toml-fmt = craneLib.taploFmt {
            src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
            # taplo arguments can be further customized below as needed
            # taploExtraArgs = "--config ./taplo.toml";
          };

          # Audit dependencies
          my-workspace-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };

          # Audit licenses
          my-workspace-deny = craneLib.cargoDeny {
            inherit src;
          };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on other crate derivations
          # if you do not want the tests to run twice
          my-workspace-nextest = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });

          # Ensure that cargo-hakari is up to date
          my-workspace-hakari = craneLib.mkCargoDerivation {
            inherit src;
            pname = "my-workspace-hakari";
            cargoArtifacts = null;
            doInstallCargoArtifacts = false;

            buildPhaseCargoCommand = ''
              cargo hakari generate --diff  # workspace-hack Cargo.toml is up-to-date
              cargo hakari manage-deps --dry-run  # all workspace crates depend on workspace-hack
              cargo hakari verify
            '';

            nativeBuildInputs = [
              pkgs.cargo-hakari
            ];
          };
        };

        packages = {
          inherit charmi;
          inherit cq_term;
          inherit game_core;
          inherit n_dit;
        } // lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
          my-workspace-llvm-coverage = craneLibLLvmTools.cargoLlvmCov (commonArgs // {
            inherit cargoArtifacts;
          });
        };

        apps = {
          # my-cli = flake-utils.lib.mkApp {
          #   drv = my-cli;
          # };
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

          # Extra inputs can be added here; cargo and rustc are provided by default.
          packages = [
            pkgs.cargo-hakari
          ];
        };
      });
}
