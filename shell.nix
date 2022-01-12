{ rust_overlay ? import (fetchTarball
  # From 2022-01-05
  "https://github.com/oxalica/rust-overlay/archive/84c58400556c1c5fa796cbc3215ba5bbd3bd848f.tar.gz")
, pkgs ? import (fetchTarball
  "https://github.com/NixOS/nixpkgs/archive/refs/tags/21.11.tar.gz") {
    overlays = [ rust_overlay ];
  } }:
let
  rust = pkgs.rust-bin.beta.latest.default.override {
    extensions = [ "rust-src" ];
  };

  inputs = import ./inputs.nix { inherit pkgs rust; };

in pkgs.mkShell { inherit (inputs) nativeBuildInputs buildInputs; }
