{ ... }: {
  perSystem = { crane, ... }:
    let
      ucs01-relay = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/ucs01-relay";
      };
      ucs00-pingpong = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/ucs00-pingpong";
      };
    in
    {
      packages = ucs01-relay.packages // ucs00-pingpong.packages;
      checks = ucs01-relay.checks // ucs00-pingpong.checks;
    };
}
