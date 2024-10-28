_: {
  perSystem =
    {
      crane,
      lib,
      dbg,
      pkgs,
      ensure-wasm-client-type,
      mkCi,
      ...
    }:
    let
      mkEthLc =
        chain-spec:
        crane.buildWasmContract {
          crateDirFromRoot = "light-clients/ethereum-light-client";
          features = [ (pkgs.lib.strings.toLower chain-spec) ];
          checks = [
            (file_path: ''
              ${ensure-wasm-client-type {
                inherit file_path;
                type = "Ethereum${chain-spec}";
              }}
            '')
          ];
        };

      minimal = mkEthLc "Minimal";
      mainnet = mkEthLc "Mainnet";

    in
    {
      packages = minimal.packages // mainnet.packages;
      checks = minimal.checks // mainnet.checks;
    };
}
