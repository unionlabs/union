{ ... }: {
  perSystem = { crane, ... }:
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
    in
    {
      packages = ucs02-nft.packages // ucs01-relay.packages // ucs00-pingpong.packages;
      checks = ucs02-nft.checks // ucs01-relay.checks // ucs01-relay-api.checks // ucs00-pingpong.checks;
    };
}
