{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.naersk.url = "github:nix-community/naersk";
  inputs.naersk.inputs.nixpkgs.follows = "nixpkgs";
  outputs =
    { self, nixpkgs, naersk }:
    let
      inherit (nixpkgs) lib;
      systems = lib.platforms.linux; # Only support linux
      forEachSystem = fn: lib.genAttrs systems (system: fn system nixpkgs.legacyPackages.${system});
    in
    {
      packages = forEachSystem (
        system: pkgs: rec {
          barnacle = pkgs.callPackage ./nix/package.nix { naersk = pkgs.callPackage naersk {}; };
          default = barnacle;
        }
      );

      devShells = forEachSystem (
        system: pkgs: {
          default = import ./nix/shell.nix {
            inherit pkgs;
            inherit (self.packages.${system}) barnacle;
          };
        }
      );
    };
}
