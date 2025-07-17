{ pkgs, ... }:
{
  voyager,
  voyager-module-plugins,
}:
let
  getPlugin = plugin: pkgs.lib.getExe' voyager-module-plugins plugin;

  mkModulePlugin = {plugin, info ? {}, config ? {}}: {
    enabled = true;
    path = getPlugin plugin;
    info = info;
    config = config;
  };
in
{
  name = "e2e-devnet-voyager";
  settings = {
    stack-size = 20971520;
    log-level = "voyager_plugin_packet_filter=trace,beacon=debug,cosmos_client=debug,concurrent_keyring=debug,cosmos_client=debug,arbitrum=debug,voyager_consensus_module_cometbls=info,voyager_message::rpc::server=info,voyager=debug,info";
    log-format = "json";
    runtime-max-secs = 3600;
  };
  package = voyager;
  modules = {
    state = [
      mkModulePlugin {
        plugin = "voyager-state-module-cosmos-sdk";
        info = {
          chain_id = "union-devnet-1";
          ibc_spec_id = "1.0.0";
        };
        config = {
          rpc_url = "http://0.0.0.0:26657";
        };
      }
      mkModulePlugin {
        plugin = "voyager-state-module-cosmos-sdk-union";
        info = {
          chain_id = "union-devnet-1";
          ibc_spec_id = "ibc-union";
        };
        config = {
          rpc_url = "http://0.0.0.0:26657";
          ibc_host_contract_address = "union1nk3nes4ef6vcjan5tz6stf9g8p08q2kgqysx6q5exxh89zakp0msq5z79t";
        };
      }
      mkModulePlugin {
        plugin = "voyager-state-module-ethereum";
        info = {
          chain_id = "32382";
          ibc_spec_id = "ibc-union";
        };
        config = {
          ibc_handler_address = "0xed2af2aD7FE0D92011b26A2e5D1B4dC7D12A47C5";
          rpc_url = "http://0.0.0.0:8545";
        };
      }
    ];
  };
}
