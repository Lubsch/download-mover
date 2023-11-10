{
  inputs.naersk = {
    url = "github:nix-community/naersk";
    inputs.nixpkgs.follows = "nixpkgs";
  };
  inputs.fenix = {
    url = "github:nix-community/fenix";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, fenix, naersk, nixpkgs }:
  let
    mapSystems = f: builtins.mapAttrs f nixpkgs.legacyPackages;
  in {

    packages = mapSystems (system: pkgs:
    let
      toolchain = with fenix.packages.${system}; combine [
        minimal.rustc
        minimal.cargo
        targets.x86_64-unknown-linux-musl.latest.rust-std
      ];
      naersk' = naersk.lib.${system}.override {
        cargo = toolchain;
        rustc = toolchain;
      };
    in {
      default = naersk'.buildPackage {
        nativeBuildInputs = [ pkgs.pkgsStatic.stdenv.cc ];
        CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
        CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
        src = ./.;
      };
    });

    devShells = mapSystems (_: pkgs: { 
      default = pkgs.mkShell {
        packages = with pkgs; [
          cargo
          rustc
          clippy
        ];
      };
    });

  };
}
