{ ... }: {
  perSystem = { crane, ... }:
    let
      cw20-ics20 = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/cw20-ics20";
      };
      ucs00-pingpong = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/ucs00-pingpong";
      };
    in
    {
      packages = cw20-ics20.packages // ucs00-pingpong.packages;
      checks = cw20-ics20.checks // ucs00-pingpong.checks;
    };
}
