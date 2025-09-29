_: {
  perSystem =
    {
      pkgs,
      system,
      pkgsUnstableSolana,
      rust,
      dbg,
      crane,
      ...
    }:
    let

      platform-tools = dbg (
        pkgs.stdenv.mkDerivation {
          pname = "platform-tools";
          version = "1.51";
          src = platform-tools-stripped;
          nativeBuildInputs = [ pkgs.autoPatchelfHook ];
          buildInputs = with pkgs; [
            zlib
            pkgs.stdenv.cc.cc
            openssl
            python310
            ncurses
            libxml2
            libedit
            xz
          ];

          postFixup = ''
            patchelf --replace-needed libedit.so.2 libedit.so $out/llvm/lib/liblldb.so.19.1.7-rust-dev
          '';

          installPhase = ''
            mkdir -p $out;
            cp -r $src/llvm $out;
            cp -r $src/rust $out;
            chmod 0755 -R $out;
          '';
        }
      );

      platform-tools-versions = {
        "x86_64-linux" = pkgs.fetchurl {
          url = "https://github.com/anza-xyz/platform-tools/releases/download/v1.51/platform-tools-linux-x86_64.tar.bz2";
          sha256 = "sha256-qdMVf5N9X2+vQyGjWoA14PgnEUpmOwFQ20kuiT7CdZc=";
        };
        "aarch64-linux" = pkgs.fetchurl {
          url = "https://github.com/anza-xyz/platform-tools/releases/download/v1.51/platform-tools-linux-aarch64.tar.bz2";
          sha256 = "sha256-4oHGs4Mg5Y726Cf2ymucSRSCX47eKc/C89qjhYW3YLs=";
        };
      };

      platform-tools-stripped = pkgs.runCommand "platform-tools-stripped" { } ''
        mkdir -p $out
        tar --strip-components=0 -xjf ${platform-tools-versions.${system}} -C $out;
      '';

      cargo-solana = pkgs.stdenv.mkDerivation {
        pname = "cargo-solana";
        version = "0.0.0";

        src = pkgsUnstableSolana.solana-cli;

        nativeBuildInputs = [
          pkgs.pkg-config
          rust.toolchains.dev
          platform-tools
        ];

        installPhase = ''
          mkdir -p $out
          cp -r $src/* $out
          chmod -R +w $out
          mkdir -p $out/bin/platform-tools-sdk/sbf/dependencies/platform-tools
          cp -r ${platform-tools}/llvm $out/bin/platform-tools-sdk/sbf/dependencies/platform-tools/llvm
          cp -r ${platform-tools}/rust $out/bin/platform-tools-sdk/sbf/dependencies/platform-tools/rust
        '';
      };

      solana-ibc =
        (crane.buildWorkspaceMember "solana/ibc" {
          cargoBuildRustToolchain = "${platform-tools}/rust";
          cargoBuildExtraArgs = "--target sbpf-solana-solana";
        }).ibc-union-solana;

      # solana-ibc = pkgs.stdenv.mkDerivation {
      #   name = "cargo-solana";
      #   buildInputs = [
      #     pkgs.makeWrapper
      #     rust.toolchains.dev
      #   ];
      #   src = ./ibc;
      #   installPhase = ''
      #     mkdir -p $out
      #     cp -r ${pkgsUnstableSolana.solana-cli}/* $out
      #     ls -la
      #     $out/bin/cargo-build-sbf --sbf-sdk $out/bin/platform-tools-sdk/sbf
      #   '';
      # };

    in
    {
      packages = {
        inherit cargo-solana solana-ibc;
      };

    };
}
