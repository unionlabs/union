{ pkgs }:
{
  name,
  package,
}:
pkgs.stdenv.mkDerivation {
  inherit name;
  src = package;
  buildPhase = ''
    cp -r $src $out
  '';
  installPhase = ''
    cp -r $src .
  '';
  doCheck = false;
}
