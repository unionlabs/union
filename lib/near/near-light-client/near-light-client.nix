_: {
  perSystem =
    {
      self',
      lib,
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
      near-light-client = crane.buildWasmContract "near/near-light-client" {
        extraBuildInputs = [
          pkgs.pkg-config
          pkgs.openssl
          pkgs.perl
          pkgs.gnumake
        ];
        extraNativeBuildInputs = [ pkgs.clang ];
      };
    in
    {
      inherit (near-light-client) packages;
    };
}
