{
  description = "An image converter written in rust";

  inputs = {
    utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      ...
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        toolchain = pkgs.rustPlatform;
        lib = nixpkgs.lib;
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      in
      rec {
        packages = {
          rimi = toolchain.buildRustPackage {
            pname = "rimi";

            version = cargoToml.package.version;

            src = ./.;

            cargoLock.lockFile = ./Cargo.lock;

            meta = {
              description = "Simple image manipulation tool.";
              homepage = "htpps://github.com/Daru-san/rimi";
              maintainers = [ lib.maintainers.daru-san ];
              license = [ lib.licenses.mit ];
              mainProgram = "rimi";
            };
          };
          default = packages.rimi;
        };
        apps = {
          rimi = utils.lib.mkApp { drv = packages.default; };
          default = apps.rimi;
        };
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            (with toolchain; [
              cargo
              rustc
              rustLibSrc
            ])
            clippy
            rustfmt
            pkg-config
          ];
          RUST_SRC_PATH = "${toolchain.rustLibSrc}";
        };
      }
    );
}
