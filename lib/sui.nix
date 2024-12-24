_: {
  perSystem =
    {
      self',
      lib,
      pkgs,
      system,
      rust,
      crane,
      config,
      ...
    }:
    let
      throwBadSystem = throw "sui cannot be built on system `${system}`";

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

      rustToolchain = rust.mkNightly {
        channel = "1.78.0";
        targets = [ CARGO_BUILD_TARGET ];
      };

      craneLib = crane.lib.overrideToolchain rustToolchain;

      sui = craneLib.buildPackage rec {
        pname = "sui";
        version = "testnet-v1.38.0";

        buildInputs = [
          pkgs.pkg-config
          pkgs.openssl
          pkgs.libarchive
          config.treefmt.build.programs.rustfmt
        ] ++ (lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ]);

        nativeBuildInputs = [ pkgs.clang ];

        src = builtins.fetchGit {
          url = "https://github.com/MystenLabs/sui";
          ref = "refs/tags/testnet-v1.38.0";
          rev = "120cb4e60fdf79272621cd018e256909733f7823";
        };

        doCheck = false;

        cargoExtraArgs = "--release";

        LIBCLANG_PATH = "${pkgs.llvmPackages_14.libclang.lib}/lib";

        CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";

      };

      suiApp = pkgs.writeShellApplication {
        name = "sui";
        runtimeInputs = [ sui ];
        text = ''
          export PATH=${sui}/bin:$PATH
          exec "$@"
        '';
      };
    in
    {
      packages = {
        inherit sui suiApp;
      };
    };
}
