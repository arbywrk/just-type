{
  description = "flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "rustfmt"
          ];
        };

        linkedLibs = with pkgs; [
          vulkan-loader
          libGL

          # Wayland
          wayland
          libxkbcommon

          # X11
          libx11
          libxcursor
          libxrandr
          libxi

          gtk4
          glib

          dbus
          zenity

          mupdf

          python3
        ];

        nativeBuildInputs = with pkgs; [
          pkg-config
          gtk4.dev
          clang
          llvmPackages.libclang
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = linkedLibs;
          nativeBuildInputs = nativeBuildInputs ++ [ rustToolchain ];

          # Makes wgpu/winit find the .so files at runtime without installing system-wide
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath linkedLibs;

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        };
      }
    );
}
