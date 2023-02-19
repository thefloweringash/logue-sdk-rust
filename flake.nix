{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.flake-utils.follows = "utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
      in
      {
        devShell = with pkgs; mkShell {
          buildInputs = [
            cargo
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
              targets = [ "thumbv7em-none-eabihf" ];
            })
            gcc-arm-embedded
            rustfmt
            rust-analyzer
            pre-commit
            rustPackages.clippy
            zip
            unzip
          ];

          HOST_CC = "${gcc-arm-embedded}/bin/arm-none-eabi-gcc";
          HOST_OBJCOPY = "${gcc-arm-embedded}/bin/arm-none-eabi-objcopy";
        };
      });
}
