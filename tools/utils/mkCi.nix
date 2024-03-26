{ pkgs }:
isCi: drv:
let
  passthru = {
    ci = isCi;
  };
in
pkgs.lib.lazyDerivation {
  derivation = drv;
  inherit passthru;
}
