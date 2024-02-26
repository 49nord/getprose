{ rust_overlay ? import (fetchTarball
  # From 2022-02-22
  "https://github.com/oxalica/rust-overlay/archive/e0626adabd5ea461f80b1b11390da2a6575adb30.tar.gz")
, pkgs ? import (fetchTarball
  "https://github.com/NixOS/nixpkgs/archive/refs/tags/21.11.tar.gz") {
    overlays = [ rust_overlay ];
  } }:
let
  rust = pkgs.rust-bin.stable."1.65.0".default.override {
    extensions = [ "rust-src" ];
  };

  inputs = import ./inputs.nix { inherit pkgs rust; };

in pkgs.mkShell { inherit (inputs) nativeBuildInputs buildInputs; }
