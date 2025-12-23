_: {
  perSystem =
    {
      self',
      mkCrane,
      pkgs,
      dbg,
      gitRev,
      ensureAtRepositoryRoot,
      ...
    }:
    let
      crane = mkCrane {
        root = ../.;
        # gitRev = "";
        inherit gitRev;
      };

      # CosmmWasm Contracts

      cw-account = crane.buildWasmContract "cosmwasm/cw-account" { };
      cw-escrow-vault = crane.buildWasmContract "cosmwasm/cw-escrow-vault" {
        # doesn't use bls precompiles, so the miscompilation is not an issue
        buildWithOz = true;
      };
      cw-unionversal-token = crane.buildWasmContract "cosmwasm/cw-unionversal-token" {
        # doesn't use bls precompiles, so the miscompilation is not an issue
        buildWithOz = true;
      };
      lst = crane.buildWasmContract "cosmwasm/lst" { };
      lst-staker = crane.buildWasmContract "cosmwasm/lst-staker" { };
      proxy-account-factory = crane.buildWasmContract "cosmwasm/proxy-account-factory" { };
      manager = crane.buildWasmContract "cosmwasm/gatekeeper" { };
      cw20-base = crane.buildWasmContract "cosmwasm/cw20-base" { };
      cw20-wrapped-tokenfactory = crane.buildWasmContract "cosmwasm/cw20-wrapped-tokenfactory" { };
      ibc-union = crane.buildWasmContract "cosmwasm/core" { };
      multicall = crane.buildWasmContract "cosmwasm/multicall" { };
      on-zkgm-call-proxy = crane.buildWasmContract "cosmwasm/on-zkgm-call-proxy" { };
      cw20-token-minter = crane.buildWasmContract "cosmwasm/cw20-token-minter" { };
      osmosis-tokenfactory-token-minter =
        crane.buildWasmContract "cosmwasm/osmosis-tokenfactory-token-minter"
          { };

      # Get the full deployments object for a chain.
      getDeployment =
        ucs04-chain-id:
        (builtins.fromJSON (builtins.readFile ../deployments/deployments.json)).${ucs04-chain-id};

      # Get a deployed contract address by name on a chain.
      getDeployedContractAddress =
        ucs04-chain-id: name:
        (pkgs.lib.lists.findSingle ({ value, ... }: value.name == name)
          (throw "no deployment found for ${name} on ${ucs04-chain-id}")
          (throw "many deployments found for ${name} on ${ucs04-chain-id}")
          (pkgs.lib.attrsToList (getDeployment ucs04-chain-id).contracts)
        ).name;

      bytecode-base = pkgs.stdenv.mkDerivation {
        name = "base-bytecode";
        dontUnpack = true;
        src = ../cosmwasm/deployer/base-bytecode.wat;
        buildInputs = [ pkgs.binaryen ];
        buildPhase = ''
          wasm-as $src -o $out
        '';
        meta = {
          description = "The raw bytecode used in the deterministic instantiate2 address derivation.";
        };
      };

      inherit (crane.buildWorkspaceMember "cosmwasm/deployer" { }) cosmwasm-deployer;

      # CosmWasm Networks
      #
      # ucs04-chain-id : The UCS04 Chain ID of this chain.
      # name           : A Unique identifier for this chain. This will be used in the nix derivation
      #                  outputs (i.e. .#cosmwasm-scripts.<name>.<script>)
      # rpc-url        : A CometBFT JSON-RPC url for this chain.
      # deployer-key   : A bash expression that resolves to an 0x-prefixed private key. This is the
      #                  key that will be used for *new* deployments; only for address derivation.
      # ops-key        : A bash expression that resolves to an 0x-prefixed private key. This is the
      #                  key that will be used for all operations *after* the initial deployment of
      #                  a contract (i.e. storing new code).
      # gas-config     : The gas parameters for transaction submission for this chain. See the docs
      #                  on `mk-gas-args` for more information.
      # bech32-prefix  : The bech32 prefix for this chain.
      # apps           : A map of IBC app name to the deployment configuration. See the docs on
      #                  `ucs03-configs` for more information.
      # lightclients   : Lightclients that are to be deployed on this chain.
      networks = [
        rec {
          ucs04-chain-id = "union.union-devnet-1";
          name = "union-devnet";
          rpc-url = "http://localhost:26657";
          # alice from the devnet keyring
          deployer-key = "0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f";
          ops-key = deployer-key;
          gas-config = {
            type = "feemarket";
            max_gas = 100000000;
            gas_multiplier = 1.4;
          };
          bech32-prefix = "union";
          apps = {
            ucs03 = (ucs03-configs.cw20 cw20-base) // {
              rate_limit_disabled = true;
            };
          };
          lightclients = [
            "trusted-mpt"
            # "sui"
          ];
        }
        {
          ucs04-chain-id = "union.union-testnet-10";
          name = "union-testnet-10";
          rpc-url = "https://rpc.rpc-node.union-testnet-10.union.build";
          # rpc-url = "https://union-testnet-rpc.polkachu.com";
          deployer-key = ''"$(op item get deployer --vault union-testnet-10 --field cosmos-deployer-key --reveal)"'';
          ops-key = ''"$(op item get ops --vault union-testnet-10 --field cosmos-ops-key --reveal)"'';
          gas-config = {
            type = "feemarket";
          };
          apps = {
            ucs03 = (ucs03-configs.cw20 cw20-base) // {
              rate_limit_disabled = true;
            };
          };
          bech32-prefix = "union";
          lightclients = [
            "arbitrum"
            "base"
            "bob"
            "berachain"
            "ethereum"
            "trusted-mpt"
            "ethermint"
            "tendermint-bls"
            "parlia"
            "sui"
            "state-lens-ics23-mpt"
          ];
          u = "union1uuuuuuuuu9un2qpksam7rlttpxc8dc76mcphhsmp39pxjnsvrtcqvyv57r";
          eu = "union1eueueueu9var4yhdruyzkjcsh74xzeug6ckyy60hs0vcqnzql2hq0lxc2f";
          lst = true;
          on-zkgm-call-proxy = true;
        }
        {
          ucs04-chain-id = "union.union-1";
          name = "union";
          rpc-url = "https://rpc.rpc-node.union-1.union.build";
          deployer-key = ''"$(op item get deployer --vault union-testnet-10 --field cosmos-deployer-key --reveal)"'';
          ops-key = ''"$(op item get ops --vault union-testnet-10 --field cosmos-ops-key --reveal)"'';
          gas-config = {
            type = "feemarket";
            max_gas = 10000000;
            gas_multiplier = 1.4;
          };
          apps = {
            ucs03 = ucs03-configs.cw20 cw20-base;
          };
          bech32-prefix = "union";
          lightclients = [
            "arbitrum"
            "bob"
            "berachain"
            "ethereum"
            "trusted-mpt"
            "ethermint"
            "base"
            "tendermint-bls"
            "state-lens-ics23-mpt"
            "parlia"
          ];
          u = "union1uuuuuuuuu9un2qpksam7rlttpxc8dc76mcphhsmp39pxjnsvrtcqvyv57r";
          eu = "union1eueueueu9var4yhdruyzkjcsh74xzeug6ckyy60hs0vcqnzql2hq0lxc2f";
          lst = true;
          on-zkgm-call-proxy = false;
        }
        {
          ucs04-chain-id = "osmosis.osmosis-devnet-1";
          name = "osmosis-devnet";
          rpc-url = "http://localhost:26857";
          deployer-key = "0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f";
          gas-config = {
            type = "fixed";
            gas_price = "0.05";
            gas_denom = "uosmo";
            gas_multiplier = "1.1";
            max_gas = 60000000;
          };
          apps = {
            ucs03 = ucs03-configs.osmosis-tokenfactory // {
              rate_limit_disabled = true;
            };
          };
          bech32-prefix = "osmo";
          lightclients = [
            "cometbls"
            "tendermint"
            "state-lens-ics23-mpt"
          ];
        }
        {
          ucs04-chain-id = "osmosis.osmo-test-5";
          name = "osmosis-testnet";
          rpc-url = "https://osmosis-testnet-rpc.polkachu.com";
          deployer-key = ''"$(op item get deployer --vault union-testnet-10 --field cosmos-deployer-key --reveal)"'';
          ops-key = ''"$(op item get ops --vault union-testnet-10 --field cosmos-ops-key --reveal)"'';
          gas-config = {
            type = "fixed";
            gas_price = "0.1";
            gas_denom = "uosmo";
            gas_multiplier = "1.2";
            max_gas = 40000000;
          };
          apps = {
            ucs03 = ucs03-configs.osmosis-tokenfactory // {
              rate_limit_disabled = true;
            };
          };
          bech32-prefix = "osmo";
          lightclients = [
            "cometbls"
            "tendermint"
            "state-lens-ics23-mpt"
          ];
        }
        {
          ucs04-chain-id = "osmosis.osmosis-1";
          name = "osmosis";
          rpc-url = "https://osmosis-rpc.polkachu.com";
          deployer-key = ''"$(op item get deployer --vault union-testnet-10 --field cosmos-deployer-key --reveal)"'';
          ops-key = ''"$(op item get ops --vault union-testnet-10 --field cosmos-ops-key --reveal)"'';
          gas-config = {
            type = "osmosis-eip1559-feemarket";
            max_gas = 60000000;
            gas_multiplier = "1.2";
            base_fee_multiplier = "1.4";
            denom = "uosmo";
          };
          apps = {
            ucs03 = ucs03-configs.osmosis-tokenfactory;
          };
          bech32-prefix = "osmo";
          lightclients = [
            "cometbls"
            "tendermint"
            "state-lens-ics23-mpt"
          ];
        }
        {
          ucs04-chain-id = "babylon.bbn-test-6";
          name = "babylon-testnet";
          rpc-url = "https://babylon-testnet-rpc.polkachu.com";
          deployer-key = ''"$(op item get deployer --vault union-testnet-10 --field cosmos-deployer-key --reveal)"'';
          ops-key = ''"$(op item get ops --vault union-testnet-10 --field cosmos-ops-key --reveal)"'';
          gas-config = {
            type = "fixed";
            gas_price = "0.003";
            gas_denom = "ubbn";
            gas_multiplier = "1.1";
            max_gas = 10000000;
          };
          apps = {
            ucs03 = (ucs03-configs.cw20 cw20-wrapped-tokenfactory) // {
              rate_limit_disabled = true;
            };
          };
          bech32-prefix = "bbn";
          lightclients = [
            "cometbls"
            "tendermint"
            "trusted-mpt"
            "state-lens-ics23-mpt"
            "state-lens-ics23-ics23"
          ];
          u = "bbn1uuuuuuuuu9un2qpksam7rlttpxc8dc76mcphhsmp39pxjnsvrtcqnrn5rr";
          eu = "bbn1eueueueu9var4yhdruyzkjcsh74xzeug6ckyy60hs0vcqnzql2hqscechf";
        }
        {
          ucs04-chain-id = "babylon.bbn-1";
          name = "babylon";
          rpc-url = "https://babylon-rpc.polkachu.com";
          deployer-key = ''"$(op item get deployer --vault union-testnet-10 --field cosmos-deployer-key --reveal)"'';
          ops-key = ''"$(op item get ops --vault union-testnet-10 --field cosmos-ops-key --reveal)"'';
          gas-config = {
            type = "fixed";
            gas_price = "0.003";
            gas_denom = "ubbn";
            gas_multiplier = "1.1";
            max_gas = 10000000;
          };
          apps = {
            ucs03 = ucs03-configs.cw20 cw20-wrapped-tokenfactory;
          };
          bech32-prefix = "bbn";
          lightclients = [
            "cometbls"
            "tendermint"
            "trusted-mpt"
            "state-lens-ics23-mpt"
          ];
        }
        {
          ucs04-chain-id = "xion.xion-testnet-2";
          name = "xion-testnet";
          rpc-url = "https://rpc.xion-testnet-2.burnt.com";
          deployer-key = ''"$(op item get deployer --vault union-testnet-10 --field cosmos-deployer-key --reveal)"'';
          ops-key = ''"$(op item get ops --vault union-testnet-10 --field cosmos-ops-key --reveal)"'';
          gas-config = {
            type = "fixed";
            gas_price = "0.002";
            gas_denom = "uxion";
            gas_multiplier = "1.5";
            max_gas = 60000000;
          };
          apps = {
            ucs03 = (ucs03-configs.cw20 cw20-base) // {
              rate_limit_disabled = true;
            };
          };
          bech32-prefix = "xion";
          lightclients = [
            "cometbls"
            "tendermint"
            "state-lens-ics23-mpt"
            "state-lens-ics23-ics23"
          ];
        }
        rec {
          ucs04-chain-id = "intento.intento-dev-1";
          name = "intento-devnet";
          rpc-url = "https://rpc-devnet.intento.zone/";
          deployer-key = ''"$(op item get intento-devnet-deployer --vault union-testnet-10 --field private-key --reveal)"'';
          ops-key = deployer-key;
          gas-config = {
            type = "fixed";
            gas_price = "0.0015";
            gas_denom = "uinto";
            gas_multiplier = "1.4";
            max_gas = 60000000;
          };
          apps = {
            ucs03 = (ucs03-configs.cw20 cw20-base) // {
              rate_limit_disabled = true;
            };
          };
          bech32-prefix = "into";
          lightclients = [
            "cometbls"
            "tendermint"
            "state-lens-ics23-mpt"
          ];
        }
      ];

      # name => { dir, client-type, features? }
      all-lightclients = {
        base = {
          dir = "base";
          client-type = "base";
        };
        bob = {
          dir = "bob";
          client-type = "bob";
        };
        arbitrum = {
          dir = "arbitrum";
          client-type = "arbitrum";
        };
        berachain = {
          dir = "berachain";
          client-type = "berachain";
        };
        cometbls = {
          dir = "cometbls";
          client-type = "cometbls";
        };
        ethereum = {
          dir = "ethereum";
          client-type = "ethereum";
        };
        trusted-mpt = {
          dir = "trusted-mpt";
          client-type = "trusted/evm/mpt";
        };
        ethermint = {
          dir = "ethermint";
          client-type = "ethermint";
        };
        tendermint = {
          dir = "tendermint";
          client-type = "tendermint";
        };
        tendermint-bls = {
          dir = "tendermint";
          client-type = "tendermint";
          features = [ "bls" ];
        };
        state-lens-ics23-mpt = {
          dir = "state-lens-ics23-mpt";
          client-type = "state-lens/ics23/mpt";
        };
        state-lens-ics23-ics23 = {
          dir = "state-lens-ics23-ics23";
          client-type = "state-lens/ics23/ics23";
        };
        sui = {
          dir = "sui";
          client-type = "sui";
        };
        parlia = {
          dir = "parlia";
          client-type = "parlia";
        };
        starknet = {
          dir = "starknet";
          client-type = "starknet";
        };
        attested = {
          dir = "attested";
          client-type = "attested";
        };
      };

      all-apps = {
        # TODO: Revive
        # ucs00-pingpong = {
        #   name = "ucs00";
        # };
        ucs03-zkgm = {
          name = "ucs03";
        };
      };

      # Gas configurations supported by cosmwasm-deployer.
      #
      # Fixed                     : Use a fixed configuration, with the specified gas denom, price,
      #                             multiplier, and max gas.
      # Feemarket                 : Query gas prices from the feemarket module. Both max gas and
      #                             multiplier are optional.
      # Osmosis EIP1559 Feemarket : Query gas prices from osmosis' eip1559 feemarket module, with
      #                             the specified denom and base fee multiplier. Both the max gas
      #                             and multiplier are optional.
      mk-gas-args =
        config@{ type, ... }:
        {
          fixed =
            {
              gas_denom,
              gas_multiplier,
              gas_price,
              max_gas,
            }:
            " --gas fixed --gas-price ${toString gas_price} --gas-denom ${toString gas_denom} --gas-multiplier ${toString gas_multiplier} --max-gas ${toString max_gas} ";
          feemarket =
            {
              max_gas ? null,
              gas_multiplier ? null,
            }:
            " --gas feemarket "
            + (pkgs.lib.optionalString (max_gas != null) " --max-gas ${toString max_gas} ")
            + (pkgs.lib.optionalString (
              gas_multiplier != null
            ) " --gas-multiplier ${toString gas_multiplier} ");
          osmosis-eip1559-feemarket =
            {
              max_gas ? null,
              gas_multiplier ? null,
              base_fee_multiplier,
              denom,
            }:
            " --gas osmosis-eip1559-feemarket "
            + " --base-fee-multiplier ${base_fee_multiplier} "
            + " --fee-denom ${denom} "
            + (pkgs.lib.optionalString (max_gas != null) " --max-gas ${toString max_gas} ")
            + (pkgs.lib.optionalString (
              gas_multiplier != null
            ) " --gas-multiplier ${toString gas_multiplier} ");
        }
        .${type}
          (builtins.removeAttrs config [ "type" ]);

      # Supported configurations for the ucs03-zkgm deployment.
      ucs03-configs = {
        cw20 = cw20-impl: {
          type = "cw20";
          path = "${(mk-app "ucs03-zkgm").release}";
          cw_account_path = "${cw-account.release}";
          token_minter_path = "${cw20-token-minter.release}";
          token_minter_config = {
            cw20 = {
              cw20_impl = "${cw20-impl.release}";
            };
          };
          rate_limit_disabled = false;
        };
        osmosis-tokenfactory = {
          type = "osmosis-tokenfactory";
          rate_limit_disabled = false;
          path = "${(mk-app "ucs03-zkgm").release}";
          cw_account_path = "${cw-account.release}";
          token_minter_path = "${osmosis-tokenfactory-token-minter.release}";
          token_minter_config = {
            osmosis_tokenfactory = { };
          };
        };
      };

      get-git-rev =
        {
          ucs04-chain-id,
          rpc-url,
          ...
        }:
        let
          # minified version of the protos found in https://github.com/CosmWasm/wasmd/tree/2e748fb4b860ee109123827f287949447f2cded7/proto/cosmwasm/wasm/v1
          cosmwasmProtoDefs = pkgs.writeTextDir "/cosmwasm.proto" ''
            syntax = "proto3";
            package cosmwasm;

            message QueryContractInfoRequest {
              string address = 1;
            }

            message QueryContractInfoResponse {
              ContractInfo contract_info = 2;
            }

            message ContractInfo {
              uint64 code_id = 1;
            }

            message QueryCodeRequest {
              uint64 code_id = 1;
            }

            message QueryCodeResponse {
              bytes data = 2;
            }

            message QuerySmartContractStateRequest {
              string address = 1;
              bytes query_data = 2;
            }

            message QuerySmartContractStateResponse {
              bytes data = 1;
            }
          '';
        in
        pkgs.writeShellApplication {
          name = "get-git-rev";
          runtimeInputs = [
            self'.packages.embed-commit-verifier
            pkgs.buf
            pkgs.xxd
            pkgs.curl
          ];
          text = ''
            embed-commit-verifier extract <(curl -L \
              --silent \
              '${rpc-url}/abci_query?path=%22/cosmwasm.wasm.v1.Query/Code%22&data=0x'"$(
                buf \
                  convert \
                  ${cosmwasmProtoDefs}/cosmwasm.proto \
                  --type=cosmwasm.QueryCodeRequest \
                  --from=<(
                    echo "{\"code_id\":$(
                      curl -L \
                        --silent \
                        '${rpc-url}/abci_query?path=%22/cosmwasm.wasm.v1.Query/ContractInfo%22&data=0x'"$(
                          buf \
                            convert \
                            ${cosmwasmProtoDefs}/cosmwasm.proto \
                            --type=cosmwasm.QueryContractInfoRequest \
                            --from=<(echo "{\"address\":\"$1\"}")#format=json \
                            | xxd -c 0 -ps
                        )" \
                        | jq .result.response.value -r \
                        | base64 -d \
                        | buf \
                          convert \
                          ${cosmwasmProtoDefs}/cosmwasm.proto \
                          --type=cosmwasm.QueryContractInfoResponse \
                          --from=-#format=binpb \
                        | jq '.contractInfo.codeId | tonumber'
                    )}"
                  )#format=json \
                  | xxd -c 0 -ps
                )" \
                | jq .result.response.value -r \
                | base64 -d \
                | buf \
                  convert \
                  ${cosmwasmProtoDefs}/cosmwasm.proto \
                  --type=cosmwasm.QueryCodeResponse \
                  --from=-#format=binpb \
                | jq .data -r \
                | base64 -d)
          '';
          meta = {
            description = "Extract the embedded git rev from a contract on ${ucs04-chain-id}.";
            longDescription = "All of our deployed wasm binaries have the current git revision embedded at compile time. This script will fetch the wasm code of the provided contract and extract the embedded git rev. See the `embed-commit` crate for more information.";
          };
        };

      deploy =
        args@{
          ucs04-chain-id,
          name,
          rpc-url,
          gas-config,
          deployer-key,
          permissioned ? false,
          ...
        }:
        pkgs.writeShellApplication {
          name = "${name}-deploy-full";
          runtimeInputs = [ cosmwasm-deployer ];
          text = ''
            PRIVATE_KEY=${deployer-key} \
            RUST_LOG=info \
              cosmwasm-deployer \
              deploy-full \
              --contracts ${chain-contracts-config-json args} \
              ${if permissioned then "--permissioned " else ""} \
              --rpc-url ${rpc-url} \
              --manager ${getDeployedContractAddress ucs04-chain-id "manager"} \
              ${mk-gas-args gas-config} "$@"
          '';
          meta = {
            description = "Deploy the full union IBC stack on ${ucs04-chain-id}.";
            longDescription = "Deploy the full union IBC stack on ${ucs04-chain-id}. The manager must first be deployed with `.#cosmwasm-scripts.${ucs04-chain-id}.deploy-manager`.";
          };
        };

      deploy-manager =
        {
          ucs04-chain-id,
          name,
          rpc-url,
          gas-config,
          deployer-key,
          ...
        }:
        pkgs.writeShellApplication {
          name = "deploy-contract-${name}";
          runtimeInputs = [
            cosmwasm-deployer
          ];
          text = ''
            PRIVATE_KEY=${deployer-key} \
            RUST_LOG=info \
              cosmwasm-deployer \
              deploy-manager \
              --bytecode ${manager.release} \
              --rpc-url ${rpc-url} \
              ${mk-gas-args gas-config} "$@"
          '';
          meta = {
            description = "Deploy the manager on ${ucs04-chain-id}.";
            longDescription = "Deploy the manager contract on ${ucs04-chain-id}. This is a prerequisite for deploying the full stack, as all of the contracts we deploy are access managed.";
          };
        };

      whitelist-relayers =
        {
          name,
          ucs04-chain-id,
          rpc-url,
          gas-config,
          ops-key,
          ...
        }:
        pkgs.writeShellApplication {
          name = "${name}-whitelist-relayers";
          runtimeInputs = [ cosmwasm-deployer ];
          text = ''
            PRIVATE_KEY=${ops-key} \
            RUST_LOG=info \
              cosmwasm-deployer \
              tx \
              whitelist-relayers \
              --manager ${(getDeployment ucs04-chain-id).manager} \
              --rpc-url ${rpc-url} \
              ${mk-gas-args gas-config} "$@"
          '';
        };

      set-bucket-config =
        {
          name,
          ucs04-chain-id,
          rpc-url,
          gas-config,
          ops-key,
          ...
        }:
        pkgs.writeShellApplication {
          name = "${name}-set-bucket-config";
          runtimeInputs = [ cosmwasm-deployer ];
          text = ''
            PRIVATE_KEY=${ops-key} \
            RUST_LOG=info \
              cosmwasm-deployer \
              tx \
              set-bucket-config \
              --rpc-url ${rpc-url} \
              --ucs03-address ${getDeployedContractAddress ucs04-chain-id "protocols/ucs03"} \
              ${mk-gas-args gas-config} "$@"
          '';
        };

      chain-contracts-config-json =
        {
          ucs04-chain-id,
          lightclients,
          apps,
          ...
        }:
        pkgs.writeText "${ucs04-chain-id}.contracts-config.json" (
          builtins.toJSON {
            core = ibc-union.release;
            lightclient = pkgs.lib.mapAttrs' (
              name: { client-type, ... }: pkgs.lib.nameValuePair client-type (mk-lightclient name)
            ) (pkgs.lib.getAttrs lightclients all-lightclients);
            app = apps;
            # escrow_vault = cw-escrow-vault.release;
          }
        );

      chain-deployed-contracts-json =
        {
          ucs04-chain-id,
          lightclients,
          apps,
          ...
        }:
        pkgs.writeText "${ucs04-chain-id}.deployed-contracts.json" (
          builtins.toJSON {
            lightclient = map ({ client-type, ... }: client-type) (
              builtins.filter ({ name, ... }: builtins.elem name lightclients) all-lightclients
            );
            app = builtins.attrNames apps;
          }
        );

      chain-migration-scripts =
        args@{
          lightclients,
          apps,
          ops-key,
          ucs04-chain-id,
          rpc-url,
          gas-config,
          ...
        }:
        (builtins.listToAttrs (
          map (
            lc:
            let
              name = "migrate-lightclient-${lc}";
            in
            {
              inherit name;
              value = pkgs.writeShellApplication {
                name = "${args.name}-${name}";
                runtimeInputs = [
                  cosmwasm-deployer
                ];
                text = ''
                  PRIVATE_KEY=${ops-key} \
                  RUST_LOG=info \
                    cosmwasm-deployer \
                    upgrade \
                    --rpc-url ${rpc-url} \
                    --address ${(getDeployedContractAddress ucs04-chain-id "lightclients/${all-lightclients.${lc}.client-type}")} \
                    --new-bytecode ${(mk-lightclient lc).release} \
                      ${mk-gas-args gas-config} "$@"
                '';
              };
            }
          ) lightclients
        ))
        // (builtins.listToAttrs (
          map (
            app:
            let
              name = "migrate-app-${app}";
            in
            {
              inherit name;
              value = pkgs.writeShellApplication {
                name = "${args.name}-${name}";
                runtimeInputs = [
                  cosmwasm-deployer
                  pkgs.jq
                ];
                text = ''
                  PRIVATE_KEY=${ops-key} \
                  RUST_LOG=info \
                    cosmwasm-deployer \
                    migrate \
                    --rpc-url ${rpc-url} \
                    --address ${getDeployedContractAddress ucs04-chain-id "core"} \
                    --new-bytecode ${(mk-app app).release} \
                    ${mk-gas-args gas-config} "$@"
                '';
              };
            }
          ) (builtins.attrNames apps)
        ))
        // (
          let
            name = "migrate-core";
          in
          {
            ${name} = pkgs.writeShellApplication {
              name = "${args.name}-${name}";
              runtimeInputs = [
                cosmwasm-deployer
              ];
              text = ''
                PRIVATE_KEY=${ops-key} \
                RUST_LOG=info \
                  cosmwasm-deployer \
                  upgrade \
                  --rpc-url ${rpc-url} \
                  --address ${(getDeployedContractAddress ucs04-chain-id).core.address} \
                  --new-bytecode ${ibc-union.release} \
                  ${mk-gas-args gas-config} "$@"
              '';
            };
          }
        );

      deploy-contract =
        {
          ucs04-chain-id,
          name,
          rpc-url,
          gas-config,
          deployer-key,
          ...
        }:
        pkgs.writeShellApplication {
          name = "deploy-contract-${name}";
          runtimeInputs = [
            cosmwasm-deployer
          ];
          text = ''
            PRIVATE_KEY=${deployer-key} \
            RUST_LOG=info \
              cosmwasm-deployer \
              deploy-contract \
              --rpc-url ${rpc-url} \
              ${mk-gas-args gas-config} "$@"
          '';
          meta = {
            description = "Deploy a contract on ${ucs04-chain-id}.";
            longDescription = "Deploy a contract on ${ucs04-chain-id} via the bytecode-base deterministic address pattern. The bytecode must be specified.";
          };
        };

      migrate-contract =
        {
          ucs04-chain-id,
          name,
          rpc-url,
          gas-config,
          ops-key,
          ...
        }:
        pkgs.writeShellApplication {
          name = "${name}-migrate-contract";
          runtimeInputs = [ cosmwasm-deployer ];
          text = ''
            PRIVATE_KEY=${ops-key} \
            RUST_LOG=info \
              cosmwasm-deployer \
              migrate \
              --rpc-url ${rpc-url} \
              --address "''${1:?address must be set (first argument to this script))}" \
              --new-bytecode "''${2:?new bytecode path must be set (second argument to this script))}" \
              ${mk-gas-args gas-config} \
              "''${@:3}"
          '';
          meta = {
            description = "Deploy a contract on ${ucs04-chain-id}.";
            longDescription = "Deploy a contract on ${ucs04-chain-id} via the bytecode-base deterministic address pattern. The bytecode must be specified.";
          };
        };

      setup-roles =
        {
          ucs04-chain-id,
          name,
          rpc-url,
          gas-config,
          ...
        }:
        pkgs.writeShellApplication {
          name = "${name}-setup-roles";
          runtimeInputs = [
            cosmwasm-deployer
            ibc-union-contract-addresses
          ];
          text = ''
            RUST_LOG=info \
              cosmwasm-deployer \
              setup-roles \
              --rpc-url ${rpc-url} \
              --manager ${(getDeployedContractAddress ucs04-chain-id).manager} \
              --addresses <(ibc-union-contract-addresses ${(getDeployment ucs04-chain-id).deployer}) \
              ${mk-gas-args gas-config} \
              "$@"
          '';
          meta = {
            description = "Setup access management roles on ${ucs04-chain-id}.";
            longDescription = "Setup access management roles for the core stack on ${ucs04-chain-id}. This must be run after the full deployment with `.#cosmwasm-scripts.${ucs04-chain-id}.deploy`.";
          };
        };

      ibc-union-contract-addresses = pkgs.writeShellApplication {
        name = "ibc-union-contract-addresses";
        runtimeInputs = [ cosmwasm-deployer ];
        text = ''
          cosmwasm-deployer \
            addresses \
            ${
              pkgs.lib.strings.concatStrings (
                pkgs.lib.mapAttrsToList (_: { client-type, ... }: " --lightclient ${client-type}") all-lightclients
              )
            } \
            ${
              pkgs.lib.strings.concatStrings (map (a: " --${all-apps.${a}.name}") (builtins.attrNames all-apps))
            } \
            --deployer "''${1:?deployer must be set (first argument to this script)}" ''${2+--output $2} 
        '';
      };

      mk-lightclient =
        name:
        let
          lc = all-lightclients.${name};
        in
        crane.buildWasmContract "cosmwasm/lightclient/${lc.dir}" {
          features = lc.features or null;
        };

      mk-app =
        dir:
        crane.buildWasmContract "cosmwasm/app/${dir}" {
          # none of our apps use bls precompiles, so the miscompilation is not an issue
          buildWithOz = true;
        };

      update-deployments-json =
        {
          name,
          rpc-url,
          ucs04-chain-id,
          lightclients,
          apps,
          lst ? false,
          u ? null,
          eu ? null,
          on-zkgm-call-proxy ? false,
          ...
        }:
        pkgs.writeShellApplication {
          name = "update-deployments-json-${name}";
          runtimeInputs = [
            self'.packages.update-deployments
          ];
          text = ''
            ${ensureAtRepositoryRoot}

            RUST_LOG=info update-deployments \
              "deployments/deployments.json" \
              ${ucs04-chain-id} \
              --rpc-url ${rpc-url} \
              ${
                pkgs.lib.concatMapStringsSep " " (
                  lc: "--lightclient ${all-lightclients.${lc}.client-type}"
                ) lightclients
              } \
              ${if apps ? ucs03 then "--ucs03 --ucs03-minter ${apps.ucs03.type}" else ""} \
              ${if apps ? ucs00 then "--ucs00" else ""} \
              ${if lst then "--lst" else ""} \
              ${if on-zkgm-call-proxy then "--on-zkgm-call-proxy" else ""} \
              ${pkgs.lib.optionalString (eu != null) "--eu ${eu}"} \
              ${pkgs.lib.optionalString (u != null) "--u ${u}"}
          '';
        };
    in
    {
      packages =
        {
          inherit
            bytecode-base
            cw20-base
            cw20-wrapped-tokenfactory
            cosmwasm-deployer
            cw20-token-minter
            osmosis-tokenfactory-token-minter
            ibc-union
            multicall
            on-zkgm-call-proxy
            cw-account
            cw-escrow-vault
            cw-unionversal-token
            lst
            lst-staker
            proxy-account-factory
            ibc-union-contract-addresses
            ;
          cosmwasm-scripts =
            {
              update-deployments-json = pkgs.writeShellApplication {
                name = "update-deployments-json";
                text =
                  let
                    deployments = pkgs.lib.filterAttrs (_: deployment: deployment.ibc_interface == "ibc-cosmwasm") (
                      builtins.fromJSON (builtins.readFile ../deployments/deployments.json)
                    );
                    getNetwork =
                      ucs04chainId:
                      pkgs.lib.lists.findSingle (network: network.ucs04-chain-id or null == ucs04chainId)
                        (throw "no network found with ucs04 chain id ${ucs04chainId}")
                        (throw "many networks with ucs04 chain id ${ucs04chainId} found")
                        networks;
                  in
                  pkgs.lib.concatMapStringsSep "\n\n" (ucs04ChainId: ''
                    echo "updating ${ucs04ChainId}"
                    ${pkgs.lib.getExe
                      self'.packages.cosmwasm-scripts.${(getNetwork ucs04ChainId).name}.update-deployments-json
                    }
                  '') (builtins.attrNames deployments);
              };
            }
            // (pkgs.mkRootDrv "cosmwasm-scripts" (
              builtins.listToAttrs (
                map (chain: {
                  inherit (chain) name;
                  value = pkgs.mkRootDrv chain.name (
                    {
                      chain-contracts-config-json = chain-contracts-config-json chain;
                      chain-deployed-contracts-json = chain-deployed-contracts-json chain;
                      deploy = deploy chain;
                      deploy-manager = deploy-manager chain;
                      update-deployments-json = update-deployments-json chain;
                      get-git-rev = get-git-rev chain;
                      whitelist-relayers = whitelist-relayers chain;
                      set-bucket-config = set-bucket-config chain;
                      deploy-contract = deploy-contract chain;
                      migrate-contract = migrate-contract chain;
                      setup-roles = setup-roles chain;
                    }
                    // (chain-migration-scripts chain)
                  );
                }) networks
              )
            ));
        }
        //
          # all light clients
          (pkgs.lib.mapAttrs' (
            name: _:
            let
              lc = mk-lightclient name;
            in
            {
              name = lc.passthru.packageName;
              value = lc;
            }
          ) all-lightclients)
        //
          # all apps
          (pkgs.lib.mapAttrs' (n: _v: rec {
            name = value.passthru.packageName;
            value = mk-app n;
          }) all-apps);
    };
}
