{
    pkgs ? import <nixpkgs> { },
    barnacle ? pkgs.callPackage ./package.nix { },
}:
let
    inherit (pkgs) writeShellScriptBin lib;
in
pkgs.mkShell {
    inputsFrom = [ barnacle ];

    # Rust development tools
    packages = with pkgs; [
        bacon
        cargo-i18n
        cargo-info
        cargo-tarpaulin
        cargo-watch
        lldb
        rust-analyzer
        rustPackages.clippy
        rustc
        rustfmt

        # Useful shell Aliases as "packages"
        (writeShellScriptBin "rmshare" ''
            rm -rf ~/.local/share/barnacle
        '')

        (writeShellScriptBin "rmdb" ''
            rm -rf ~/.local/state/barnacle
        '')

        (writeShellScriptBin "nuke" ''
            rm -rf ~/.local/share/barnacle
            rm -rf ~/.local/state/barnacle
            rm -rf ~/.config/barnacle
        '')
    ];

    # Ensure runtime dependencies are available
    LD_LIBRARY_PATH = lib.makeLibraryPath [
        pkgs.wayland
        pkgs.libxkbcommon
        pkgs.fontconfig
        pkgs.libGL
        pkgs.dbus
    ];
}
