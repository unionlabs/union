{ ... }: {
  perSystem = { crane, ensure-wasm-client-size, ... }:
    let
      ucs01-relay = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/ucs01-relay";
        checks = [
          (file_path: ''
            ${ensure-wasm-client-size {
              inherit file_path;
              max_size = 800 * 1024;
            }}
          '')
        ];
      };
      ucs01-relay-api = crane.buildWorkspaceMember {
        crateDirFromRoot = "cosmwasm/ucs01-relay-api";
      };
      ucs00-pingpong = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/ucs00-pingpong";
        checks = [
          (file_path: ''
            ${ensure-wasm-client-size {
              inherit file_path;
              max_size = 800 * 1024;
            }}
          '')
        ];
      };
    in
    {
      packages = ucs01-relay.packages // ucs00-pingpong.packages;
      checks = ucs01-relay.checks // ucs01-relay-api.checks // ucs00-pingpong.checks;
    };
}
