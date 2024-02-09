{ nixpkgs ? import <nixpkgs> {} }:
let
  nixpkgs-unstable = nixpkgs.fetchFromGitHub {
    owner  = "NixOS";
    repo   = "nixpkgs";
    rev    = "6832d0d99649db3d65a0e15fa51471537b2c56a6";
    sha256 = "sha256-0etC/exQIaqC9vliKhc3eZE2Mm2wgLa0tj93ZF/egvM=";
  };
  pkgs = import nixpkgs-unstable {};
in
pkgs.mkShell {
    buildInputs = with pkgs;[
      rustup
      wasm-bindgen-cli
      wasm-pack
    ];
    RUST_BACKTRACE = 1;
    
    shellHook = "alias compile='wasm-pack build --target web'";
}
