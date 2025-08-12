{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      utils,
      naersk,
      ...
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = callPackage naersk { };

        inherit (pkgs)
          mkShell
          rustPlatform
          callPackage
          lib
          ;
      in
      {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = mkShell {
          packages = with pkgs; [
            # Tools
            bacon
            cargo
            diesel-cli
            cargo-info
            rustPackages.clippy
            rustfmt

            # Dependencies
            fuse-overlayfs
            libarchive
            openssl
            pkg-config
            rustc

            # Slint
            fontconfig
            xorg.libxcb
            wayland
            libxkbcommon
            libGL
          ];
          env = {
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
            LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${
              with pkgs;
              lib.makeLibraryPath [
                wayland
                libxkbcommon
                fontconfig
                libGL
              ]
            }";
          };
        };
      }
    );
}
