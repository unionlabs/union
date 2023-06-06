{ ... }: {
  perSystem = { pkgs, self', crane, system, ... }:
    let
      CARGO_BUILD_TARGET =
        if system == "aarch64-linux" then "aarch64-unknown-linux-musl"
        else if system == "x86_64-linux" then "x86_64-unknown-linux-musl"
        else if system == "aarch64-darwin" then "aarch64-apple-darwin"
        else if system == "x86_64-darwin" then "x86_64-apple-darwin"
        else throwBadSystem;

      throwBadSystem = throw "libwasmvm cannot be built on system ${system}";

      wasmvm = pkgs.fetchFromGitHub {
        owner = "CosmWasm";
        repo = "wasmvm";
        rev = "v1.2.3"; # wasmd 0.40
        hash = "sha256-GscUMJ0Tkg77S9IYA9komyKKoa1AyVXSSaU8hw3ZNwk=";
      };
    in
    {
      packages.libwasmvm =
        (crane.withBuildTarget CARGO_BUILD_TARGET).buildPackage (
          {
            name = "libwasmvm";
            version = "1.2.3";
            src = "${wasmvm}/libwasmvm";
            doCheck = false;
            inherit CARGO_BUILD_TARGET;
          } // (if pkgs.stdenv.isLinux then {
            cargoBuildCommand = "cargo build --release --example=muslc";
            installPhase = ''
              mkdir -p $out/lib
              mv target/${CARGO_BUILD_TARGET}/release/examples/libmuslc.a $out/lib/libwasmvm.${builtins.head (pkgs.lib.strings.splitString "-" system)}.a
            '';
          } else if pkgs.stdenv.isDarwin then {
            # non-static dylib build on macOS
            cargoBuildCommand = "cargo build --release";
            installPhase = ''
              mkdir -p $out/lib
              mv target/${CARGO_BUILD_TARGET}/release/deps/libwasmvm.dylib $out/lib/libwasmvm.dylib 
            '';
          } else throwBadSystem)
        );

    };
}
