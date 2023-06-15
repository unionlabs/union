{ pkgs }:
{ name
, package
, ...
}:
pkgs.stdenv.mkDerivation {
  name = name;
  src = package;
  buildPhase = ''
    cp -r $src $out
  '';
  installPhase = ''
    cp -r $src .
  '';
  doCheck = false;
}
