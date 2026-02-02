{
  lib,
  rustPlatform,

  pkg-config,
  makeWrapper,

  fuse-overlayfs,
  libarchive,
  openssl,
  cargo-tarpaulin,
  cargo-i18n,

  wayland,
  libxkbcommon,
  fontconfig,
  libGL,
  dbus,

  naersk,
}:
naersk.buildPackage {
  pname = "barnacle";
  version = "0";

  src = ../.;

  nativeBuildInputs = [
    pkg-config
    makeWrapper
  ];

  buildInputs = [
    fuse-overlayfs # NOTE: Might only be necessary during runtime
    libarchive
    openssl
  ];
  cargoHash = "sha256-6I0JKaeVJU5ROPToItwFwEO+UPr5OtFvY8ebJXKm0Yc=";

  postInstall = ''
    wrapProgram $out/bin/barnacle-gui \
      --prefix LD_LIBRARY_PATH : ${
        lib.makeLibraryPath [
          wayland
          libxkbcommon
          fontconfig
          libGL
          dbus
        ]
      }
  '';

  meta = with lib; {
    homepage = "https://github.com/poperigby/barnacle";
    description = "Fast, powerful mod manager for Linux";
    mainProgram = "barnacle-gui";
    license = licenses.gpl3Only;
    platforms = platforms.linux;
    maintainers = [
      maintainers.kruziikrel13
      maintainers.poperigby
    ];
  };
}
