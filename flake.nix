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

        armBinutils = pkgs.pkgsCross.arm-embedded.buildPackages.binutils-unwrapped;

        minimodem = pkgs.callPackage ./minimodem {};
      in
      {
        devShell = with pkgs; mkShell {
          buildInputs = [
            cargo
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
              targets = [
                "thumbv7em-none-eabihf"
                "wasm32-unknown-unknown"
              ];
            })
            armBinutils
            rustfmt
            rust-analyzer
            pre-commit
            rustPackages.clippy
            zip
            unzip
            cargo-show-asm
            llvm_12
            libxml2
            wabt
            rustfilt
            minimodem
          ];

          HOST_OBJCOPY = "${armBinutils}/bin/arm-none-eabi-objcopy";
        };
      });
}
