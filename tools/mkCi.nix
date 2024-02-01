{ pkgs, lib, ...}:
{isCi, drv}:
let
  inherit (lib) types mkOption mdDoc;
  passthru = {
    ci = isCi;
  };
in
lib.lazyDerivation {
  derivation = drv;
  inherit passthru;
}
