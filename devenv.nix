{
  pkgs,
  lib,
  ...
}:

{
  languages.rust = {
    enable = true;
    mold.enable = true;
    components = [
      "cargo"
      "rustc"
      "rustfmt"
      "rust-analyzer"
      "clippy"
    ];
  };

  packages = with pkgs; [
    # Tools
    bacon
    cargo-info
    cargo-tarpaulin

    # Dependencies
    fuse-overlayfs
    libarchive
    openssl
    pkg-config

    # GUI
    fontconfig
    xorg.libxcb
    wayland
    libxkbcommon
    libGL
  ];

  env = {
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
}
