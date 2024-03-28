dir:
assert builtins.isPath dir;
let
  mkIncludePath = p: "${dir}/${p}/default.nix";
in
builtins.filter
  builtins.pathExists
  (map mkIncludePath (builtins.attrNames (builtins.readDir dir)))
