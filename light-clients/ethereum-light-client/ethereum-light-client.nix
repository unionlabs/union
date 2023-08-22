{ ... }: {
  perSystem = { crane, lib, dbg, ... }:
    let
      mkEthLc = chain-spec: crane.buildWasmContract {
        crateDirFromRoot = "light-clients/ethereum-light-client";
        features = [ chain-spec ];
        additionalTestSrcFilter = path: _:
          (lib.hasPrefix "light-clients/ethereum-light-client/src/test" path)
          && (lib.strings.hasSuffix ".json" path);
      };

      minimal = mkEthLc "minimal";
      mainnet = mkEthLc "mainnet";
    in
    {
      packages = minimal.packages // mainnet.packages;
      checks = minimal.checks // mainnet.checks;
    };
}
