{
  description = "A recreation of a favorite childhood flash game";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
      };
    in {
      formatter = pkgs.alejandra;

      packages.default = with pkgs; with lib.fileset; stdenv.mkDerivation {
        pname = "n_dit";
        version = "0.1";
        buildInputs = [pkgs.cargo];
        src = toSource {
          root = ./.;
          fileset = unions [
            ./assets # Might remove in long term
            ./Cargo.lock
            ./Cargo.toml
            ./src
            ./charmi/Cargo.toml
            ./charmi/src
            ./charmi_bevy/Cargo.toml
            ./charmi_bevy/src
            ./charmi_macros/Cargo.toml
            ./charmi_macros/src
            ./cq_term/Cargo.toml
            ./cq_term/src
            ./game_core/Cargo.toml
            ./game_core/src
          ];
        };
        buildPhase = ''
          cargo build;
          mv target/debug $out;
        '';
      };

      devShells.default = with pkgs; mkShell rec {
        nativeBuildInputs = [
          pkg-config
          mold
          clang
          cargo
        ];
        buildInputs = [
          udev
          alsa-lib-with-plugins
          # alsa-lib.dev
          # vulkan-loader
          # xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr # To use the x11 feature
          # libxkbcommon wayland # To use the wayland feature
        ];
        LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
      };
    }
  );
}
