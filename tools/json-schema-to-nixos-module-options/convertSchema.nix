{
  self',
  config,
  pkgs,
  buildSchema,
  schemaPath,
  name,
  ensureAtRepositoryRoot,
}:
assert pkgs.lib.isDerivation buildSchema;
assert builtins.isString schemaPath;
assert builtins.isString name;
let
  nixosModuleOptions =
    pkgs.runCommand "${name}-nixosModuleOptions.nix"
      {
        buildInputs = [
          config.treefmt.build.wrapper
          self'.packages.json-schema-to-nixos-module-options
        ];
      }
      ''
        json-schema-to-nixos-module-options ${buildSchema} > out.nix

        touch treefmt.nix # for treefmt to find the root

        treefmt --no-cache out.nix

        mv out.nix $out
      '';
in
{
  checks = {
    "${name}-nixos-module-options-up-to-date" = pkgs.stdenv.mkDerivation {
      name = "${name}-module-up-to-date";
      dontUnpack = true;
      src = nixosModuleOptions;
      doCheck = true;
      checkPhase = ''
        diff "$src" ${../../. + ("/" + schemaPath)}

        touch $out
      '';
    };
  };
  packages = {
    "generate-${name}-nixos-options" = pkgs.writeShellApplication {
      name = "generate-${name}-nixos-options";
      runtimeInputs = [
        self'.packages.json-schema-to-nixos-module-options
      ];
      text = ''
        ${ensureAtRepositoryRoot}

        cp --no-preserve=mode ${nixosModuleOptions} ${schemaPath}
      '';
    };
  };
}
