{ pkgs ? import <nixpkgs> {} }: let
  fenix = import (pkgs.fetchFromGitHub {
    owner = "nix-community";
    repo = "fenix";
    rev = "90d5a3f59c7a3a4056775c17a6058776e800d21b";
    sha256 = "DJprLijphC/v03WbymWK5ORRXl36/tIJCjMda6/2Nwk=";
  }) {};

  rust-toolchain = fenix.complete.withComponents [
    "cargo"
    "rustc"
    "clippy"
    "rustfmt"
  ];
in pkgs.mkShell {
  nativeBuildInputs = [
    rust-toolchain
    pkgs.cargo-expand
  ];
}
