{
  description = "wasm-pack setup";

  inputs = {
    nixpkgs = { url = "github:nixos/nixpkgs/nixos-unstable"; };
    rust-overlay = { url = "github:oxalica/rust-overlay"; };
  };

  outputs = { nixpkgs, rust-overlay, ... }:
    let system = "x86_64-linux";
    in {
      devShell.${system} = let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlay ];
        };
      in (({ pkgs, ... }:
        pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ pkg-config openssl.dev ];
          buildInputs = with pkgs; [
            cargo
            cargo-watch
			      openssl
            nodejs
            wasm-pack
			      wasm-bindgen-cli
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
              targets = [ "wasm32-unknown-unknown" ];
            })
          ];

          shellHook = "";
        }) { pkgs = pkgs; });
    };
}
