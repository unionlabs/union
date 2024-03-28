{ inputs, ... }: {
  perSystem =
    { pkgs, unstablePkgs, crane, dbg, system, rust, mkCi, nix-filter, ... }:
    let
      CARGO_BUILD_TARGET = rust.staticBuildTarget system;

      rustToolchain = rust.mkNightly { target = CARGO_BUILD_TARGET; };

      BIOME_VERSION = "1.6.3";

      biome = (crane.lib.overrideToolchain rustToolchain).buildPackage {
        inherit CARGO_BUILD_TARGET BIOME_VERSION;

        pname = "biome";
        version = BIOME_VERSION;
        src = inputs.biome;

        nativeBuildInputs = [ pkgs.pkg-config ];

        buildInputs = [ pkgs.libgit2 unstablePkgs.rust-jemalloc-sys pkgs.zlib ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin
          [ pkgs.darwin.apple_sdk.frameworks.Security ];

        nativeCheckInputs = [ pkgs.git ];

        cargoExtraArgs = "-p=biome_cli";

        doCheck = false;

        meta.mainProgram = "biome";
      };
    in
    {
      _module.args.biome = biome;
      checks.biome-lint = mkCi (system == "x86_64-linux") (pkgs.stdenv.mkDerivation {
        name = "biome-lint";
        src =
          let
            fs = unstablePkgs.lib.fileset;
            root = ../.;
          in
          fs.toSource {
            inherit root;
            fileset = fs.intersection
              (fs.difference root (fs.unions [
                ../galoisd/vendor
                ../uniond/vendor
                ../app/src/generated
              ]))
              (fs.fileFilter
                (file: (file.name != "package-lock.json") && (builtins.any file.hasExt [
                  "js"
                  "ts"
                  "cjs"
                  "mjs"
                  "jsx"
                  "tsx"
                  "d.ts"
                  "css"
                  "astro"
                  "svelte"
                  "json"
                  "jsonc"
                ]))
                root);
          };
        buildInputs = [ biome pkgs.tree ];
        doCheck = true;
        checkPhase = ''
          cd $src
          tree .
          biome lint . --error-on-warnings --verbose
          touch $out
        '';
      });
    };
}
