{
  description = "A very basic flake";

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
