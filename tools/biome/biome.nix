{ inputs, ... }: {
  perSystem =
    { pkgs, unstablePkgs, crane, dbg, system, rust, mkCi, nix-filter, ... }:
    let
      throwBadSystem = throw "libwasmvm cannot be built on system `${system}`";

      CARGO_BUILD_TARGET = if system == "aarch64-linux" then
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

      BIOME_VERSION = "1.6.2";

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
    in {
      _module.args.biome = biome;
      checks.biome-lint = mkCi (system == "x86_64") (pkgs.stdenv.mkDerivation {
        name = "biome-lint";
        src = nix-filter {
          root = ../../.;
          include = [ nix-filter.isDirectory ]
            ++ (builtins.map nix-filter.matchExt [
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
            ]);
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
