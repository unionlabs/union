_: {
  perSystem =
    {
      pkgs,
      pkgsUnstable,
      crane,
      rust,
      dbg,
      ...
    }:
    let
      craneLib = crane.lib.overrideToolchain (
        _:
        rust.mkToolchain {
          channel = "1.90.0";
        }
      );

      arch = "aarch64";

      openvm-version = "v1.4.3";

      openvm-src = pkgs.fetchFromGitHub {
        name = "openvm";
        # owner = "openvm-org";
        owner = "benluelo";
        repo = "openvm";
        rev = "${openvm-version}-patched";
        sha256 = "sha256-vpsS0QIsdy+b8WeBddNPgnsBMwV9OWI+6yTtmJmcX+0=";
      };

      cargo-openvm = craneLib.buildPackage {
        pname = "openvm";
        version = openvm-version;
        src = openvm-src;
        cargoExtraArgs = "-p cargo-openvm"; # --features aot
        doCheck = false;
        meta.mainProgram = "openvm";
        DOCS_RS = true;
      };

      fake-rustup = pkgs.writeShellApplication {
        name = "rustup";
        text = '''';
      };

      rustup-settings-toml = builtins.toFile "rustup-settings.toml" ''
        version = "12"
        default_toolchain = "stable-${arch}-unknown-linux-gnu"
        profile = "default"

        [overrides]
      '';

      rustup-multirust-config-toml = builtins.toFile "rustup-multirust-config.toml" ''
        config_version = "1"

        [[components]]
        pkg = "cargo"
        target = "${arch}-unknown-linux-gnu"
        is_extension = false

        [[components]]
        pkg = "clippy-preview"
        target = "${arch}-unknown-linux-gnu"
        is_extension = false

        [[components]]
        pkg = "rust-docs"
        target = "${arch}-unknown-linux-gnu"
        is_extension = false

        [[components]]
        pkg = "rust-std"
        target = "${arch}-unknown-linux-gnu"
        is_extension = false

        [[components]]
        pkg = "rustc"
        target = "${arch}-unknown-linux-gnu"
        is_extension = false

        [[components]]
        pkg = "rustfmt-preview"
        target = "${arch}-unknown-linux-gnu"
        is_extension = false

        [[components]]
        pkg = "rust-src"
        target = "${arch}-unknown-linux-gnu"
        is_extension = false
      '';

      rustup-multirust-channel-manifest-toml = pkgs.fetchurl {
        url = "https://static.rust-lang.org/dist/2025-08-02/channel-rust-nightly.toml";
        hash = "sha256-QnkfTssgWvuyHRH3IkYAk3IHpKi4klsOvVIN+hKsqkY=";
      };

      rustup-components = builtins.toFile "rustup-components" ''
        cargo-${arch}-unknown-linux-gnu
        rust-std-${arch}-unknown-linux-gnu
        rustc-${arch}-unknown-linux-gnu
      '';

      rustup-home = pkgs.runCommand "rustup-home" { } ''
        mkdir $out
        cp ${rustup-settings-toml} $out/settings.toml
        mkdir -p $out/toolchains/nightly-2025-08-02-${arch}-unknown-linux-gnu/
        mkdir $out/update-hashes/
        cp -rv ${
          dbg (
            rust.mkToolchain {
              targets = [ "${arch}-unknown-linux-gnu" ];
              channel = "nightly-2025-08-02";
              components = [
                "rustc"
                "cargo"
                "rust-src"
              ];
            }
          )
        }/* $out/toolchains/nightly-2025-08-02-${arch}-unknown-linux-gnu/
        chmod -R +w $out/toolchains/nightly-2025-08-02-${arch}-unknown-linux-gnu/
        ls -alh $out/toolchains/nightly-2025-08-02-${arch}-unknown-linux-gnu/lib/rustlib
        cp ${rustup-multirust-channel-manifest-toml} $out/toolchains/nightly-2025-08-02-${arch}-unknown-linux-gnu/lib/rustlib/multirust-channel-manifest.toml
        cp ${rustup-multirust-config-toml} $out/toolchains/nightly-2025-08-02-${arch}-unknown-linux-gnu/lib/rustlib/multirust-config.toml
        cp ${rustup-components} $out/toolchains/nightly-2025-08-02-${arch}-unknown-linux-gnu/lib/rustlib/components
        echo 42791f4ecb205afbb21d > $out/update-hashes/nightly-2025-08-02-${arch}-unknown-linux-gnu

      '';

      solc_0_8_19 = pkgs.gccStdenv.mkDerivation rec {
        pname = "solc";
        version = "0.8.19";
        src = pkgs.fetchurl {
          url = "https://github.com/nikitastupin/solc/raw/main/linux/aarch64/solc-v${version}";
          hash = "sha256-5cBr9y7aGlD9f5HrPXF/CkvZsMASV8PnypBfwHOspfE=";
        };
        dontUnpack = true;
        nativeBuildInputs = [
          pkgs.stdenv.cc.cc.lib
          pkgs.autoPatchelfHook
        ];
        installPhase = ''
          runHook preInstall
          mkdir -p $out/bin
          cp ${src} $out/bin/solc
          chmod +x $out/bin/solc
          runHook postInstall
        '';
        meta = {
          description = "Static binary of compiler for Ethereum smart contract language Solidity";
          homepage = "https://github.com/ethereum/solidity";
          mainProgram = "solc";
          license = pkgs.lib.licenses.gpl3;
        };
      };

      rustup =
        let
          rustup-renamed = pkgs.runCommand "rename-rustup" { } ''
            mkdir $out
            cp ${pkgs.rustup}/bin/.rustup-wrapped $out/rustup
          '';
        in
        pkgs.writeShellApplication {
          name = "rustup";
          runtimeEnv.RUSTUP_HOME = rustup-home;
          text = ''
            ${rustup-renamed}/rustup "$@"
          '';
        };
    in
    {
      packages = {
        inherit solc_0_8_19;
        cargo-openvm = pkgs.writeShellApplication {
          name = "cargo-openvm";
          runtimeInputs = [
            # fake-rustup
            cargo-openvm
            rustup
            solc_0_8_19
          ];
          runtimeEnv = {
            RUSTUP_HOME = rustup-home;
            RUSTUP_LOG = "trace";
          };
          text = ''
            cargo-openvm "$@"
          '';
        };
        rustup = rustup;
      };
    };
}
