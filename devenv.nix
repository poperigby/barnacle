{
  pkgs,
  lib,
  ...
}:

{
  languages.rust = {
    enable = true;
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
    diesel-cli
    cargo-info

    # Dependencies
    fuse-overlayfs
    libarchive
    openssl
    pkg-config

    # Slint
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
