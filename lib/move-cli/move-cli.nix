{ ... }: {
  perSystem = { self', lib, unstablePkgs, pkgs, system, config, rust, crane, stdenv, dbg, ... }:
    let
      throwBadSystem = throw "aptos cannot be built on system `${system}`";

      CARGO_BUILD_TARGET =
        if system == "aarch64-linux" then "aarch64-unknown-linux-musl"
        else if system == "x86_64-linux" then "x86_64-unknown-linux-musl"
        else if system == "aarch64-darwin" then "aarch64-apple-darwin"
        else if system == "x86_64-darwin" then "x86_64-apple-darwin"
        else throwBadSystem;

      rustToolchain = rust.mkNightly {
        channel = "1.78.0";
        targets = [ CARGO_BUILD_TARGET ];
      };

      craneLib = crane.lib.overrideToolchain rustToolchain;

      move-cli = pkgs.stdenv.mkDerivation {
        name = "move-cli";
        buildInputs = [ pkgs.makeWrapper ];
        src =
          (crane.buildWorkspaceMember {
            crateDirFromRoot = "lib/move-cli";
            extraEnv = {
              # PROTOC = "${pkgs.protobuf}/bin/protoc";
              LIBCLANG_PATH = "${pkgs.llvmPackages_14.libclang.lib}/lib";
            };
            extraBuildInputs = [ pkgs.pkg-config pkgs.openssl pkgs.perl pkgs.gnumake ];
            extraNativeBuildInputs = [ pkgs.clang ];
            extraEnv = { };
          }).packages.move-cli;
        installPhase = ''
          mkdir -p $out/bin
          cp -r $src/bin/move-cli $out/bin/move-cli
          wrapProgram $out/bin/move-cli 
        '';
        meta.mainProgram = "move-cli";
      };

    in
    {
      packages = move-cli.packages;

    };
}
