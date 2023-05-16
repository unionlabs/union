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
        craneLib.buildPackage ({
          src = "${
              pkgs.fetchFromGitHub {
                owner = "CosmWasm";
                repo = "wasmvm";
                rev = "a9e26c0e4e5a076d82556c4f44abeee2a64ff37e";
                hash = "sha256-zR47q8Z2znPigecPDmw5L4ef20/TXv8cPxaXTdJGxg0=";
              }
            }/libwasmvm";
          doCheck = false;
        } // (if pkgs.stdenv.isLinux then rec {
          CARGO_BUILD_TARGET = if system == "aarch64-linux" then "aarch64-unknown-linux-musl" else "x86_64-unknown-linux-musl";
          cargoBuildCommand = "cargo build --release --example=muslc";
          installPhase = ''
            mkdir -p $out/lib
            ls -al target/${CARGO_BUILD_TARGET}/release/examples/libmuslc.a
            mv target/${CARGO_BUILD_TARGET}/release/examples/libmuslc.a $out/lib/libwasmvm.${builtins.head (pkgs.lib.strings.splitString "-" system)}.a
          '';
        } else { }));

    };
}
