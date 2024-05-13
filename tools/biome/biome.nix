{ inputs, ... }: {
  perSystem =
    { pkgs, unstablePkgs, crane, dbg, system, rust, mkCi, nix-filter, ... }:
    let
      throwBadSystem = throw "libwasmvm cannot be built on system `${system}`";

      CARGO_BUILD_TARGET =
        if system == "aarch64-linux" then
          "aarch64-unknown-linux-musl"
        else if system == "x86_64-linux" then
          "x86_64-unknown-linux-musl"
        else if system == "aarch64-darwin" then
          "aarch64-apple-darwin"
        else if system == "x86_64-darwin" then
          "x86_64-apple-darwin"
        else
          throwBadSystem;

      rustToolchain = rust.mkNightly { target = CARGO_BUILD_TARGET; };

      BIOME_VERSION = "1.7.3";

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
      # (system == "x86_64-linux")
      checks.biome-lint = mkCi false (pkgs.stdenv.mkDerivation {
        name = "biome-lint";
        description = "Lint js,ts,jsx,tsx,d.ts,json,jsonc,astro,svelte,vue files";
        src = with unstablePkgs.lib.fileset; toSource {
          root = ../../.;
          fileset = intersection
            (difference ../../. (unions [ ../../galoisd/vendor ../../uniond/vendor ../../app/src/generated ]))
            (fileFilter
              (file: (file.name != "package-lock.json") && (builtins.any file.hasExt [
                "js"
                "ts"
                "mts"
                "cjs"
                "mjs"
                "jsx"
                "tsx"
                "vue"
                "d.ts"
                "css"
                "astro"
                "svelte"
                "json"
                "jsonc"
              ])) ../../.);
        };
        buildInputs = [ biome ];
        doCheck = true;
        checkPhase = ''
          cd $src

          biome check . \
            --verbose \
            --error-on-warnings \
            --log-level="info" \
            --log-kind="pretty" \
            --diagnostic-level="info"

          echo "biome-lint: OK"
          touch $out
        '';
      });
    };
}
