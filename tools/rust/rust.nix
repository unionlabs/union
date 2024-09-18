{ ... }: {
  perSystem = { pkgs, system, dbg, ensureAtRepositoryRoot, mkCi, ... }:
    let
      # https://rust-lang.github.io/rustup-components-history/
      nightlyVersion = "2024-09-17";
      defaultChannel = "nightly-${nightlyVersion}";

      # # hopefully if we ever use wasi this issue will be resolved: https://github.com/NixOS/nixpkgs/pull/146274
      # targetPlatformTarget = pkgs.rust.toRustTarget pkgs.targetPlatform;

      availableComponents = {
        rustc = "rustc";
        cargo = "cargo";
        rustfmt = "rustfmt";
        rust-std = "rust-std";
        rust-docs = "rust-docs";
        rust-analyzer = "rust-analyzer";
        clippy = "clippy";
        miri = "miri";
        rust-src = "rust-src";
        llvm-tools-preview = "llvm-tools-preview";
      };

      # rustSrc =
      #   let
      #     content = pkgs.rust-bin.nightly.${nightlyVersion}._manifest.pkg.rust-src.target."*";
      #     # copied from https://github.com/oxalica/rust-overlay/blob/44210df7a70dcf0a81a5919f9422b6ae589ee673/rust-overlay.nix#L123C36-L123C36
      #     mkComponentSrc = { url, sha256 }:
      #       let
      #         url' = pkgs.lib.replaceStrings [ " " ] [ "%20" ] url; # This is required or download will fail.
      #         # Filter names like `llvm-tools-1.34.2 (6c2484dc3 2019-05-13)-aarch64-unknown-linux-gnu.tar.xz`
      #         matchParenPart = builtins.match ".*/([^ /]*) [(][^)]*[)](.*)" url;
      #         name = if matchParenPart == null then "" else (pkgs.lib.elemAt matchParenPart 0) + (pkgs.lib.elemAt matchParenPart 1);
      #       in
      #       builtins.fetchurl {
      #         inherit name sha256;
      #         url = url';
      #       };
      #   in
      #   dbg (mkComponentSrc {
      #     url = content.xz_url;
      #     sha256 = content.xz_hash;
      #   });

      rustSrc = (dbg (mkToolchain {
        components = [ availableComponents.rust-src ];
      })).passthru.availableComponents.rust-src;

      mkToolchain =
        { targets ? [ ]
        , components ? [ ]
        , channel ? defaultChannel
        ,
        }:
        pkgs.rust-bin.fromRustupToolchain {
          inherit channel targets;
          # this is the easiest way to pull in the least amount possible, even though rust-std
          # isn't required for all use cases (i.e. -Z build-std, where we use rust-src instead)
          #
          # it should be possible to construct the toolchains manually, but this works for now
          profile = "minimal";
          components = pkgs.lib.checkListOfEnum
            "rustup components"
            (builtins.attrValues availableComponents)
            components
            components;
        };

      mkBuildStdToolchain =
        { targets ? [ ]
        , channel ? defaultChannel
        }:
        mkToolchain {
          inherit targets;
          components = with availableComponents; [ rustc cargo rust-src ];
        };

      mkNightly =
        { targets ? [ ]
        , channel ? defaultChannel
        }:
        mkToolchain {
          inherit targets channel;
          components = with availableComponents; [ rustc cargo rust-std clippy ];
        };
    in
    rec {
      packages.rust-home = _module.args.rust.toolchains.dev;

      packages.fetchRustStdCargoLock = mkCi false (pkgs.writeShellApplication {
        name = "fetchRustStdCargoLock";
        runtimeInputs = [ pkgs.xz ];
        text = ''
          ${ensureAtRepositoryRoot}

          # echo ${rustSrc}
          # ls -al ${rustSrc}

          cp ${rustSrc}/lib/rustlib/src/rust/library/Cargo.lock tools/rust/rust-std-Cargo.lock
        '';
      });

      _module.args.rust = {
        inherit mkBuildStdToolchain mkNightly rustSrc;

        toolchains = {
          nightly = mkNightly { };

          # for use in the devShell
          dev = (pkgs.rust-bin.nightly.${nightlyVersion}.default.override {
            extensions = builtins.attrValues availableComponents;
            targets = [ "wasm32-unknown-unknown" ];
          });
        };
      };
    };
}
