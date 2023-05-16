{ ... }: {
  perSystem = { pkgs, self', crane, system, ... }:
    {
      packages.libwasmvm =
        let
          rustToolchain = pkgs.rust-bin.stable.latest.default.override
            (if system == "aarch64-linux" then {
              targets = [ "aarch64-unknown-linux-musl" ];
            } else if system == "x86_64-linux" then {
              targets = [ "x86_64-unknown-linux-musl" ];
            } else { });
          craneLib = crane.lib.overrideToolchain rustToolchain;
        in
        craneLib.buildPackage (
          let
            CARGO_BUILD_TARGET =
              if system == "aarch64-linux" then "aarch64-unknown-linux-musl"
              else if system == "x86_64-linux" then "x86_64-unknown-linux-musl"
              else if system == "aarch64-darwin" then "aarch64-apple-darwin"
              else if system == "x86_64-darwin" then "x86_64-apple-darwin"
              else "";
          in
          {
            src = "${
              pkgs.fetchFromGitHub {
                owner = "CosmWasm";
                repo = "wasmvm";
                rev = "a9e26c0e4e5a076d82556c4f44abeee2a64ff37e";
                hash = "sha256-zR47q8Z2znPigecPDmw5L4ef20/TXv8cPxaXTdJGxg0=";
              }
            }/libwasmvm";
            doCheck = false;
            inherit CARGO_BUILD_TARGET;
          } // (if pkgs.stdenv.isLinux then {
            cargoBuildCommand = "cargo build --release --example=muslc";
            installPhase = ''
              mkdir -p $out/lib
              mv target/${CARGO_BUILD_TARGET}/release/examples/libmuslc.a $out/lib/libwasmvm.${builtins.head (pkgs.lib.strings.splitString "-" system)}.a
            '';
          } else {
            # non-static dylib build on macOS
            cargoBuildCommand = "cargo build --release";
            installPhase = ''
              mkdir -p $out/lib
              # mv target/${CARGO_BUILD_TARGET}/release/examples/libwasmvmstatic.a $out/lib/libwasmvm.${builtins.head (pkgs.lib.strings.splitString "-" system)}.a
              mv target/${CARGO_BUILD_TARGET}/release/deps/libwasmvm.dylib $out/lib/libwasmvm.dylib 
            '';
          })
        );

    };
}
