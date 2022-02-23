{ rust_overlay ? import (fetchTarball
  # From 2022-02-22
  "https://github.com/oxalica/rust-overlay/archive/84cf30277b2685efc762e4f069eeb46cf6422904.tar.gz")
, pkgs ? import (fetchTarball
  "https://github.com/NixOS/nixpkgs/archive/refs/tags/21.11.tar.gz") {
    overlays = [ rust_overlay ];
  } }:
let
  rust = pkgs.rust-bin.stable."1.56.1".default.override {
    extensions = [ "rust-src" ];
  };

  inputs = import ./inputs.nix { inherit pkgs rust; };

in pkgs.mkShell { inherit (inputs) nativeBuildInputs buildInputs; }
