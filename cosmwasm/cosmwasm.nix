{ inputs, ... }:
{
  perSystem =
    { crane, ... }:
    let
      ucs02-nft = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/ucs02-nft";
      };
      ucs00-pingpong = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/ucs00-pingpong";
      };
      cw721-base = crane.buildRemoteWasmContract {
        src = inputs.cosmwasm-nfts;
        version = inputs.cosmwasm-nfts.rev;
        package = "cw721-base@0.18.0";
        contractFileNameWithoutExt = "cw721_base";
      };
      ibc-union = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/ibc-union/core";
      };
      ibc-union-ucs03-zkgm = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/ibc-union/app/ucs03-zkgm";
      };
      multicall = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/multicall";
      };
    in
    {
      packages =
        {
          inherit cw721-base;
        }
        // ucs02-nft.packages
        // ucs00-pingpong.packages
        // ibc-union.packages
        // multicall.packages
        // ibc-union-ucs03-zkgm.packages;
      checks =
        ucs02-nft.checks
        // ucs00-pingpong.checks
        // ibc-union-ucs03-zkgm.checks;
    };
}
