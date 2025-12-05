_: {
  perSystem =
    {
      lib,
      pkgsUnstable,
      system,
      config,
      rust,
      crane,
      ...
    }:
    let
      throwBadSystem = throw "aptos cannot be built on system `${system}`";

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

      aptos = craneLib.buildPackage rec {
        pname = "movement";
        version = "001913f20f140aa8245cd55cbb492df91b6e0e0e";

        buildInputs = [
          pkgsUnstable.pkg-config
          pkgsUnstable.openssl
          pkgsUnstable.systemd
          config.treefmt.build.programs.rustfmt
          pkgsUnstable.elfutils
          pkgsUnstable.lld
          pkgsUnstable.mold
        ] ++ (lib.optionals pkgsUnstable.stdenv.isDarwin [ pkgsUnstable.darwin.apple_sdk.frameworks.Security ]);

        nativeBuildInputs = [
          pkgsUnstable.clang
        ];

        cargoExtraArgs = "-p movement";

        LIBCLANG_PATH = "${pkgsUnstable.llvmPackages_16.libclang.lib}/lib";

        CARGO_PROFILE = "cli";

        # CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";

        src = builtins.fetchGit {
          url = "https://github.com/aeryz/aptos-core";
          ref = "old-bump-tonic";
          rev = version;
        };

        doCheck = false;
      };

      movement = pkgsUnstable.writeShellApplication {
        name = "movement";
        runtimeInputs = [
          pkgsUnstable.systemd
          aptos
        ];
        text = ''
          out=$(mktemp -d)
          cp ${aptos}/bin/movement "$out"
          chmod +x "$out/movement"
          # TODO(aeryz): not having a good time but for some reason, I can't produce a static bin
          LD_LIBRARY_PATH="${
            pkgsUnstable.lib.makeLibraryPath [
              pkgsUnstable.openssl
              pkgsUnstable.systemd
              pkgsUnstable.gcc13Stdenv.cc.cc
            ]
          }" "$out/movement" "$@"
        '';
      };

      movefmt = craneLib.buildPackage rec {
        pname = "movefmt";
        version = "3201309e4cce72205994e32a4d45d1447db705e5";

        src = builtins.fetchGit {
          url = "https://github.com/movebit/movefmt";
          ref = "develop";
          rev = version;
        };

        doCheck = false;
      };

    in
    {
      packages = {
        inherit movement movefmt;
      };

    };
}
