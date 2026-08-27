{
  description = "jerekode — OpenCode-compatible AI coding agent (Rust core + optional Bun sidecar)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        version = cargoToml.workspace.package.version;

        # Full build (default Cargo features = bun-sidecar ON). System Bun is still
        # required at runtime for Bun/TS plugins and the sidecar TUI path.
        mkJerekode =
          {
            pname,
            nativeOnly ? false,
          }:
          pkgs.rustPlatform.buildRustPackage {
            inherit pname version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            cargoBuildFlags =
              [
                "-p"
                "jerekode-cli"
              ]
              ++ pkgs.lib.optionals nativeOnly [ "--no-default-features" ];
            # Skip workspace integration tests inside the Nix build (CI covers them).
            doCheck = false;
            meta = with pkgs.lib; {
              description = "OpenCode-compatible AI coding agent runtime";
              homepage = "https://github.com/JesseKoldewijn/jerekode";
              license = licenses.mit;
              mainProgram = "jerekode";
              platforms = platforms.unix ++ platforms.windows;
            };
          };

        jerekode = mkJerekode { pname = "jerekode"; };
        jerekode-native = mkJerekode {
          pname = "jerekode-native";
          nativeOnly = true;
        };
      in
      {
        packages = {
          default = jerekode;
          jerekode = jerekode;
          jerekode-native = jerekode-native;
        };

        apps.default = {
          type = "app";
          program = "${jerekode}/bin/jerekode";
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustc
            cargo
            clippy
            rustfmt
            pkg-config
          ];
          shellHook = ''
            echo "jerekode dev shell — build with: cargo build -p jerekode-cli --locked"
            echo "Native-only: cargo build -p jerekode-cli --no-default-features --locked"
          '';
        };
      }
    );
}
