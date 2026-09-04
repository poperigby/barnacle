{
    description = "Barnacle Mod Manager";

    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
        crane.url = "github:ipetkov/crane";
        flake-utils.url = "github:numtide/flake-utils";
        advisory-db = {
            url = "github:rustsec/advisory-db";
            flake = false;
        };
    };

    outputs =
        {
            self,
            nixpkgs,
            crane,
            flake-utils,
            advisory-db,
            ...
        }:
        flake-utils.lib.eachSystem
            [
                "aarch64-linux"
                "x86_64-linux"
            ]
            (
                system:
                let
                    pkgs = nixpkgs.legacyPackages.${system};
                    lib = pkgs.lib;

                    craneLib = crane.mkLib pkgs;
                    src = craneLib.cleanCargoSource ./.;

                    commonArgs = {
                        inherit src;
                        strictDeps = true;

                        nativeBuildInputs = with pkgs; [
                            pkg-config
                        ];

                        buildInputs = with pkgs; [
                            libarchive
                            openssl
                            sqlite
                        ];
                    };

                    libraryPath = lib.makeLibraryPath (
                        with pkgs;
                        [
                            dbus
                            fontconfig
                            libGL
                            libxkbcommon
                            wayland
                        ]
                    );

                    # Build *just* the cargo dependencies, so we can reuse
                    # all of that work (e.g. via cachix) when running in CI
                    cargoArtifacts = craneLib.buildDepsOnly commonArgs;

                    individualCrateArgs = commonArgs // {
                        inherit cargoArtifacts;
                        inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
                        # NB: we disable tests since we'll run them all via cargo-nextest
                        doCheck = false;
                    };

                    barnacle-cli = craneLib.buildPackage (
                        individualCrateArgs
                        // rec {
                            inherit src;

                            pname = "barnacle-cli";
                            cargoExtraArgs = "-p ${pname}";
                        }
                    );

                    barnacle-gui = craneLib.buildPackage (
                        individualCrateArgs
                        // rec {
                            inherit src;

                            pname = "barnacle-gui";
                            cargoExtraArgs = "-p ${pname}";

                            nativeBuildInputs = commonArgs.nativeBuildInputs ++ [ pkgs.makeWrapper ];

                            postInstall = ''
                                wrapProgram $out/bin/${pname} --prefix LD_LIBRARY_PATH : ${libraryPath}
                            '';

                        }
                    );
                in
                {
                    checks = {
                        # Build the crates as part of `nix flake check` for convenience
                        inherit barnacle-cli barnacle-gui;

                        # Run clippy (and deny all warnings) on the workspace source,
                        # again, reusing the dependency artifacts from above.
                        #
                        # Note that this is done as a separate derivation so that
                        # we can block the CI if there are issues here, but not
                        # prevent downstream consumers from building our crate by itself.
                        barnacle-clippy = craneLib.cargoClippy (
                            commonArgs
                            // {
                                inherit cargoArtifacts;
                                cargoClippyExtraArgs = "--all-targets -- --deny warnings";
                            }
                        );

                        barnacle-doc = craneLib.cargoDoc (
                            commonArgs
                            // {
                                inherit cargoArtifacts;
                                # This can be commented out or tweaked as necessary, e.g. set to
                                # `--deny rustdoc::broken-intra-doc-links` to only enforce that lint
                                env.RUSTDOCFLAGS = "--deny warnings";
                            }
                        );

                        # Check formatting
                        barnacle-fmt = craneLib.cargoFmt {
                            inherit src;
                        };

                        barnacle-toml-fmt = craneLib.taploFmt {
                            src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
                            # taplo arguments can be further customized below as needed
                            # taploExtraArgs = "--config ./taplo.toml";
                        };

                        # Audit dependencies
                        barnacle-audit = craneLib.cargoAudit {
                            inherit src advisory-db;
                        };

                        # Audit licenses
                        barnacle-deny = craneLib.cargoDeny {
                            inherit src;
                        };

                        # Run tests with cargo-nextest
                        # Consider setting `doCheck = false` on other crate derivations
                        # if you do not want the tests to run twice
                        barnacle-nextest = craneLib.cargoNextest (
                            commonArgs
                            // {
                                inherit cargoArtifacts;
                                partitions = 1;
                                partitionType = "count";
                                cargoNextestPartitionsExtraArgs = "--no-tests=pass";
                            }
                        );

                        # Ensure that cargo-hakari is up to date
                        barnacle-hakari = craneLib.mkCargoDerivation {
                            inherit src;
                            pname = "barnacle-hakari";
                            cargoArtifacts = null;
                            doInstallCargoArtifacts = false;

                            buildPhaseCargoCommand = ''
                                cargo hakari generate --diff  # workspace-hack Cargo.toml is up-to-date
                                cargo hakari manage-deps --dry-run  # all workspace crates depend on workspace-hack
                                cargo hakari verify
                            '';

                            nativeBuildInputs = with pkgs; [
                                cargo-hakari
                            ];
                        };
                    };

                    packages = {
                        inherit barnacle-cli barnacle-gui;

                        default = barnacle-gui;
                    };

                    apps = {
                        default = self.apps.${system}.gui;

                        gui = {
                            type = "app";
                            program = "${barnacle-gui}/bin/barnacle-gui";
                        };

                        cli = {
                            type = "app";
                            program = "${barnacle-cli}/bin/barnacle-cli";
                        };
                    };

                    devShells.default = craneLib.devShell {
                        inputsFrom = [
                            barnacle-cli
                            barnacle-gui
                        ];

                        LD_LIBRARY_PATH = libraryPath;

                        checks = self.checks.${system};
                    };
                }
            );
}
