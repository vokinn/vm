{
  description = "flake for rust projects";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      toolchain = fenix.packages.${system}.complete.withComponents [
        "cargo"
        "rustc"
        "rust-src"
        "rustfmt"
        "rust-analyzer"
        "clippy"
      ];
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = [
          toolchain
          pkgs.cargo
        ];

        RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/src";
      };
    };
}
