let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  # Look here for information about how to generate `nixpkgs-version.json`.
  #  → https://nixos.wiki/wiki/FAQ/Pinning_Nixpkgs
  pinnedVersion = builtins.fromJSON (builtins.readFile ./.nixpkgs-version.json);
  pinnedPkgs = import (builtins.fetchGit {
    inherit (pinnedVersion) url rev;

    ref = "nixos-unstable";
  }) { overlays = [ moz_overlay ]; };
  rustChannel = pinnedPkgs.rustChannelOf { channel = "1.39.0"; };
in

# This allows overriding pkgs by passing `--arg pkgs ...`
{ pkgs ? pinnedPkgs }:

with pkgs;

mkShell {
  buildInputs = [
    cacert
    libdrm
    rustChannel.cargo
    rustChannel.rust
  ];

  LD_LIBRARY_PATH="${libdrm}/lib:$LD_LIBRARY_PATH";
}
