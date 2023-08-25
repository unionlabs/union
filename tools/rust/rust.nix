{ ... }: {
  perSystem = { pkgs, system, dbg, ... }:
    let
      nightlyVersion = "2023-05-16";
      channel = "nightly-${nightlyVersion}";

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
        rust-src = "rust-src";
        llvm-tools-preview = "llvm-tools-preview";
      };

      mkToolchain =
        { target ? null
        , components ? [ ]
        ,
        }:
        pkgs.rust-bin.fromRustupToolchain {
          inherit channel;
          # this is the easiest way to pull in the least amount possible, even though rust-std
          # isn't required for all use cases (i.e. -Z build-std, where we use rust-src instead)
          #
          # it should be possible to construct the toolchains manually, but this works for now
          profile = "minimal";
          targets = if target != null then assert builtins.isString target; [ target ] else [ ];
          components = pkgs.lib.checkListOfEnum
            "rustup components"
            (builtins.attrValues availableComponents)
            components
            components;
        };

      mkBuildStdToolchain = { target ? null }:
        mkToolchain {
          inherit target;
          components = with availableComponents; [ rustc cargo rust-src ];
        };

      mkNightly = { target ? null }:
        mkToolchain {
          inherit target;
          components = with availableComponents; [ rustc cargo rust-std clippy ];
        };
    in
    {
      _module.args.rust = {
        inherit mkBuildStdToolchain mkNightly;

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
