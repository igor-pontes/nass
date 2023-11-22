{ nixpkgs ? import <nixpkgs> {} }:
let
  nixpkgs-unstable = nixpkgs.fetchFromGitHub {
    owner  = "NixOS";
    repo   = "nixpkgs";
    rev    = "1991e35a264a4e706869f2bfbf59a2d0c0377a79";
    sha256 = "sha256-KynK4AAn6LTIYcjScHDv/aaCZ50gr1piXP6cgze6ew8=";
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
