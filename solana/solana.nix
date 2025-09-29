_: {
  perSystem =
    {
      pkgs,
      pkgsUnstableSolana,
      rust,
      ...
    }:
    let

      platform-tools = pkgs.stdenv.mkDerivation {
        pname = "platform-tols";
        version = "1.48";
        src = platform-tools-stripped;
        nativeBuildInputs = [ pkgs.autoPatchelfHook ];
        buildInputs = with pkgs; [
          zlib
          pkgs.stdenv.cc.cc
          openssl
          python310
          ncurses
          libxml2
          editline
          xz
        ];

        dontAutoPatchelf = true;

        installPhase = ''
          mkdir -p $out;
          cp -r $src/llvm $out;
          cp -r $src/rust $out;
          chmod 0755 -R $out;
        '';
        
        postFixup = ''
          for bin in $out/rust/bin/cargo $out/rust/bin/rustc; do
            echo "Patching $bin"
            patchelf --set-interpreter $(cat $NIX_CC/nix-support/dynamic-linker) $bin
            patchelf --set-rpath ${pkgs.glibc}/lib:${pkgs.zlib}/lib:${pkgs.openssl}/lib:$out/rust/lib $bin
          done
        '';
      };
      
      platform-tools-stripped = pkgs.runCommand "platform-tools-stripped" {} ''
        mkdir -p $out
        tar --strip-components=0 -xjf ${
        pkgs.fetchurl {
          url = "https://github.com/anza-xyz/platform-tools/releases/download/v1.48/platform-tools-linux-x86_64.tar.bz2";
          sha256 = "sha256-qdMVf5N9X2+vQyGjWoA14PgnEUpmOwFQ20kuiT7CdZc=";
        }
        } -C $out;

        
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
          cp -r ${platform-tools}/llvm $out/bin/platform-tools-sdk/sbf/llvm
          cp -r ${platform-tools}/rust $out/bin/platform-tools-sdk/sbf/rust
        '';
      };

      solana-ibc =  pkgs.stdenv.mkDerivation {
        name = "cargo-solana";
        buildInputs = [ pkgs.makeWrapper  rust.toolchains.dev  ]; 
        src = ./ibc;
        installPhase = ''
          mkdir -p $out
          cp -r ${pkgsUnstableSolana.solana-cli}/* $out
          ls -la
          $out/bin/cargo-build-sbf --sbf-sdk $out/bin/platform-tools-sdk/sbf
        '';
      };

    in
    {
      packages = {
        inherit cargo-solana solana-ibc;
      };

    };
}
