{ types, mkOption }:
let
  definitions = {
    "#/definitions/CacheConfig" = types.submodule {
      options = {
        "capacity" = mkOption { type = types.int; };
        "time_to_idle" = mkOption { type = types.int; };
        "time_to_live" = mkOption { type = types.int; };
      };
    };
    "#/definitions/ClientBootstrapModuleInfo" = types.submodule {
      options = {
        "chain_id" = mkOption { type = types.str; };
        "client_type" = mkOption { type = types.str; };
      };
    };
    "#/definitions/ClientModuleInfo" = types.submodule {
      options = {
        "client_type" = mkOption { type = types.str; };
        "consensus_type" = mkOption { type = types.str; };
        "ibc_interface" = mkOption { type = types.str; };
        "ibc_spec_id" = mkOption { type = types.str; };
      };
    };
    "#/definitions/Config" = types.submodule {
      options = {
        "state" = mkOption { type = definitions."#/definitions/CacheConfig"; };
      };
    };
    "#/definitions/Duration" = types.submodule {
      options = {
        "nanos" = mkOption { type = types.int; };
        "secs" = mkOption { type = types.int; };
      };
    };
    "#/definitions/FinalityModuleInfo" = types.submodule {
      options = {
        "chain_id" = mkOption { type = types.str; };
        "consensus_type" = mkOption { type = types.str; };
      };
    };
    "#/definitions/ModuleConfig_for_ClientBootstrapModuleInfo" = types.submodule {
      options = {
        "config" = mkOption {
          type = types.attrs;
          default = { };
        };
        "enabled" = mkOption {
          type = types.bool;
          default = true;
        };
        "info" = mkOption { type = definitions."#/definitions/ClientBootstrapModuleInfo"; };
        "path" = mkOption { type = types.str; };
      };
    };
    "#/definitions/ModuleConfig_for_ClientModuleInfo" = types.submodule {
      options = {
        "config" = mkOption {
          type = types.attrs;
          default = { };
        };
        "enabled" = mkOption {
          type = types.bool;
          default = true;
        };
        "info" = mkOption { type = definitions."#/definitions/ClientModuleInfo"; };
        "path" = mkOption { type = types.str; };
      };
    };
    "#/definitions/ModuleConfig_for_FinalityModuleInfo" = types.submodule {
      options = {
        "config" = mkOption {
          type = types.attrs;
          default = { };
        };
        "enabled" = mkOption {
          type = types.bool;
          default = true;
        };
        "info" = mkOption { type = definitions."#/definitions/FinalityModuleInfo"; };
        "path" = mkOption { type = types.str; };
      };
    };
    "#/definitions/ModuleConfig_for_ProofModuleInfo" = types.submodule {
      options = {
        "config" = mkOption {
          type = types.attrs;
          default = { };
        };
        "enabled" = mkOption {
          type = types.bool;
          default = true;
        };
        "info" = mkOption { type = definitions."#/definitions/ProofModuleInfo"; };
        "path" = mkOption { type = types.str; };
      };
    };
    "#/definitions/ModuleConfig_for_StateModuleInfo" = types.submodule {
      options = {
        "config" = mkOption {
          type = types.attrs;
          default = { };
        };
        "enabled" = mkOption {
          type = types.bool;
          default = true;
        };
        "info" = mkOption { type = definitions."#/definitions/StateModuleInfo"; };
        "path" = mkOption { type = types.str; };
      };
    };
    "#/definitions/ModulesConfig" = types.submodule {
      options = {
        "client" = mkOption {
          type = types.listOf definitions."#/definitions/ModuleConfig_for_ClientModuleInfo";
        };
        "client_bootstrap" = mkOption {
          type = types.listOf definitions."#/definitions/ModuleConfig_for_ClientBootstrapModuleInfo";
        };
        "consensus" = mkOption {
          type = types.listOf definitions."#/definitions/ModuleConfig_for_FinalityModuleInfo";
        };
        "proof" = mkOption {
          type = types.listOf definitions."#/definitions/ModuleConfig_for_ProofModuleInfo";
        };
        "state" = mkOption {
          type = types.listOf definitions."#/definitions/ModuleConfig_for_StateModuleInfo";
        };
      };
    };
    "#/definitions/PluginConfig" = types.submodule {
      options = {
        "config" = mkOption { type = types.attrs; };
        "enabled" = mkOption {
          type = types.bool;
          default = true;
        };
        "path" = mkOption { type = types.str; };
      };
    };
    "#/definitions/ProofModuleInfo" = types.submodule {
      options = {
        "chain_id" = mkOption { type = types.str; };
        "ibc_spec_id" = mkOption { type = types.str; };
      };
    };
    "#/definitions/QueueConfig" = types.attrs;
    "#/definitions/StateModuleInfo" = types.submodule {
      options = {
        "chain_id" = mkOption { type = types.str; };
        "ibc_spec_id" = mkOption { type = types.str; };
      };
    };
    "#/definitions/VoyagerConfig" = types.submodule {
      options = {
        "cache" = mkOption { type = definitions."#/definitions/Config"; };
        "ipc_client_request_timeout" = mkOption {
          type = definitions."#/definitions/Duration";
          default = {
            "nanos" = 0;
            "secs" = 60;
          };
        };
        "metrics_endpoint" = mkOption {
          type = types.str;
          default = "http://localhost:4318";
        };
        "num_workers" = mkOption { type = types.int; };
        "optimizer_delay_milliseconds" = mkOption {
          type = types.int;
          default = 100;
        };
        "queue" = mkOption { type = definitions."#/definitions/QueueConfig"; };
        "rest_laddr" = mkOption {
          type = types.str;
          default = "0.0.0.0:7177";
        };
        "rpc_laddr" = mkOption {
          type = types.str;
          default = "0.0.0.0:7178";
        };
      };
    };
  };
in
{
  "$schema" = mkOption { type = types.nullOr types.str; };
  "equivalent_chain_ids" = mkOption {
    type = types.listOf (types.listOf types.str);
    default = [ ];
  };
  "modules" = mkOption { type = definitions."#/definitions/ModulesConfig"; };
  "plugins" = mkOption { type = types.listOf definitions."#/definitions/PluginConfig"; };
  "voyager" = mkOption { type = definitions."#/definitions/VoyagerConfig"; };
}
