_: {
  perSystem =
    {
      self',
      lib,
      unstablePkgs,
      pkgs,
      system,
      config,
      rust,
      crane,
      stdenv,
      dbg,
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
        version = "7839b9501066108cb2322ba9039120a41781a1b0";

        buildInputs = [
          pkgs.pkg-config
          pkgs.openssl
          pkgs.systemd
          config.treefmt.build.programs.rustfmt
          pkgs.elfutils
          pkgs.lld
          pkgs.mold
        ] ++ (lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ]);

        nativeBuildInputs = [
          pkgs.clang
        ];

        cargoExtraArgs = "--release";

        LIBCLANG_PATH = "${pkgs.llvmPackages_14.libclang.lib}/lib";

        src = builtins.fetchGit {
          url = "https://github.com/MystenLabs/sui";
          ref = "framework/testnet";
          rev = version;
        };

        doCheck = false;

        # Forcing static linking if necessary
        CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
      };

      suiNode = pkgs.writeShellApplication {
        name = "sui";
        runtimeInputs = [
          pkgs.systemd
          sui
        ];
        text = ''
          export LD_LIBRARY_PATH="${
            pkgs.lib.makeLibraryPath [
              pkgs.openssl
              pkgs.systemd
              pkgs.gcc13Stdenv.cc.cc
            ]
          }:$LD_LIBRARY_PATH"
          exec ${sui}/bin/sui "$@"
        '';
      };

    in
    {
      packages = {
        inherit suiNode;
      };
    };
}
