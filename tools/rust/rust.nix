{ ... }: {
  perSystem = { pkgs, ... }:
    let
      nightlyConfig = {
        channel = "nightly-2023-05-16";
        components = [ "rust-src" "rust-analyzer" ];
        profile = "default";
        targets = [ "wasm32-unknown-unknown" ];
      };

      rust-nightly = pkgs.rust-bin.fromRustupToolchain nightlyConfig;
    in
    {
      _module.args.rust = {
        nightly = rust-nightly;
        config = nightlyConfig;
      };
    };
}
