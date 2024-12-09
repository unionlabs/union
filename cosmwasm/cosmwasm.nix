{ inputs, ... }:
{
  perSystem =
    { crane, ... }:
    let
      ucs02-nft = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/ucs02-nft";
      };
      ucs01-relay = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/ucs01-relay";
      };
      ucs01-relay-api = crane.buildWorkspaceMember {
        crateDirFromRoot = "cosmwasm/ucs01-relay-api";
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
      union-ibc = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/union-ibc/core";
      };
    in
    {
      packages = {
        inherit cw721-base;
      } // ucs02-nft.packages // ucs01-relay.packages // ucs00-pingpong.packages // union-ibc.packages;
      checks = ucs02-nft.checks // ucs01-relay.checks // ucs01-relay-api.checks // ucs00-pingpong.checks;
    };
}
