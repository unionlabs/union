_: {
  perSystem =
    {
      self',
      crane,
      pkgs,
      dbg,
      system,
      ensureAtRepositoryRoot,
      ...
    }:
    let
      getDeployment =
        ucs04-chain-id:
        (builtins.fromJSON (builtins.readFile ../deployments/deployments.json)).${ucs04-chain-id};

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

      bytecode-base = pkgs.stdenv.mkDerivation {
        name = "base-bytecode";
        dontUnpack = true;
        src = ../cosmwasm/deployer/base-bytecode.wat;
        buildInputs = [ pkgs.binaryen ];
        buildPhase = ''
          wasm-as $src -o $out
        '';
      };

      inherit (crane.buildWorkspaceMember "cosmwasm/deployer" { }) cosmwasm-deployer;

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
        }
        .${type}
          (builtins.removeAttrs config [ "type" ]);

      networks = [
        {
          chain-id = "union-devnet-1";
          ucs04-chain-id = "union.union-devnet-1";
          name = "union-devnet";
          rpc_url = "http://localhost:26657";
          # alice from the devnet keyring
          private_key = "0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f";
          gas_config = {
            type = "feemarket";
            max_gas = 10000000;
            gas_multiplier = 1.4;
          };
          ucs03_type = "cw20";
          bech32_prefix = "union";
          apps = {
            ucs03 = (ucs03-configs.cw20 cw20-base) // {
              rate_limit_disabled = true;
            };
          };
          # lightclients = pkgs.lib.lists.remove "cometbls" (builtins.attrNames all-lightclients);
          lightclients = [
            # "sui"
            "trusted-mpt"
          ];
        }
        {
          chain-id = "union-testnet-10";
          ucs04-chain-id = "union.union-testnet-10";
          name = "union-testnet-10";
          rpc_url = "https://rpc.rpc-node.union-testnet-10.union.build";
          private_key = ''"$(op item get deployer --vault union-testnet-10 --field cosmos-private-key --reveal)"'';
          gas_config = {
            type = "feemarket";
          };
          apps = {
            ucs03 = (ucs03-configs.cw20 cw20-base) // {
              rate_limit_disabled = true;
            };
          };
          bech32_prefix = "union";
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
            # "movement"
            "state-lens-ics23-mpt"
            # "state-lens-ics23-smt"
          ];
        }
        {
          chain-id = "union-1";
          ucs04-chain-id = "union.union-1";
          name = "union";
          rpc_url = "https://rpc.rpc-node.union-1.union.build";
          private_key = ''"$(op item get deployer --vault union-testnet-10 --field cosmos-private-key --reveal)"'';
          gas_config = {
            type = "feemarket";
            max_gas = 10000000;
            gas_multiplier = 1.4;
          };
          apps = {
            ucs03 = ucs03-configs.cw20 cw20-base;
          };
          bech32_prefix = "union";
          lightclients = [
            "arbitrum"
            "bob"
            "berachain"
            "ethereum"
            "trusted-mpt"
            "ethermint"
            "tendermint-bls"
            # "movement"
            "state-lens-ics23-mpt"
            # "state-lens-ics23-smt"
            "parlia"
          ];
        }
        {
          chain-id = "elgafar-1";
          name = "stargaze-testnet";
          rpc_url = "https://rpc.elgafar-1.stargaze.chain.kitchen";
          private_key = ''"$1"'';
          gas_config = {
            type = "fixed";
            gas_price = "1.0";
            gas_denom = "ustars";
            gas_multiplier = "1.1";
            max_gas = 10000000;
          };
          apps = {
            ucs03 = (ucs03-configs.cw20 cw20-base) // {
              rate_limit_disabled = true;
            };
          };
          bech32_prefix = "stars";
          lightclients = [
            "cometbls"
            "tendermint"
            "state-lens-ics23-mpt"
          ];
        }
        {
          chain-id = "osmosis-devnet-1";
          name = "osmosis-devnet";
          rpc_url = "http://localhost:26857";
          private_key = "0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f";
          gas_config = {
            type = "fixed";
            gas_price = "0.05";
            gas_denom = "uosmo";
            gas_multiplier = "1.1";
            max_gas = 60000000;
          };
          apps = {
            ucs03 = ucs03-configs.osmosis_tokenfactory // {
              rate_limit_disabled = true;
            };
          };
          bech32_prefix = "osmo";
          lightclients = [
            "cometbls"
            "tendermint"
            "state-lens-ics23-mpt"
          ];
        }
        {
          chain-id = "osmo-test-5";
          ucs04-chain-id = "osmosis.osmo-test-5";
          name = "osmosis-testnet";
          rpc_url = "https://osmosis-testnet-rpc.polkachu.com";
          private_key = ''"$(op item get deployer --vault union-testnet-10 --field cosmos-private-key --reveal)"'';
          gas_config = {
            type = "fixed";
            gas_price = "0.1";
            gas_denom = "uosmo";
            gas_multiplier = "1.2";
            max_gas = 40000000;
          };
          apps = {
            ucs03 = ucs03-configs.osmosis_tokenfactory // {
              rate_limit_disabled = true;
            };
          };
          bech32_prefix = "osmo";
          lightclients = [
            "cometbls"
            "tendermint"
            "state-lens-ics23-mpt"
          ];
        }
        {
          chain-id = "osmosis-1";
          ucs04-chain-id = "osmosis.osmosis-1";
          name = "osmosis";
          rpc_url = "https://osmosis-rpc.publicnode.com:443";
          private_key = ''"$(op item get deployer --vault union-testnet-10 --field cosmos-private-key --reveal)"'';
          gas_config = {
            type = "fixed";
            gas_price = "0.005";
            gas_denom = "uosmo";
            gas_multiplier = "1.1";
            max_gas = 60000000;
          };
          apps = {
            ucs03 = ucs03-configs.osmosis_tokenfactory;
          };
          bech32_prefix = "osmo";
          lightclients = [
            "cometbls"
            "tendermint"
            "state-lens-ics23-mpt"
          ];
        }
        {
          chain-id = "bbn-test-5";
          ucs04-chain-id = "babylon.bbn-test-5";
          name = "babylon-testnet";
          rpc_url = "https://babylon-testnet-rpc.polkachu.com";
          private_key = ''"$(op item get deployer --vault union-testnet-10 --field cosmos-private-key --reveal)"'';
          gas_config = {
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
          bech32_prefix = "bbn";
          lightclients = [
            "cometbls"
            "tendermint"
            "trusted-mpt"
            "state-lens-ics23-mpt"
            "state-lens-ics23-ics23"
          ];
        }
        {
          chain-id = "bbn-1";
          ucs04-chain-id = "babylon.bbn-1";
          name = "babylon";
          rpc_url = "https://babylon-rpc.polkachu.com";
          private_key = ''"$(op item get deployer --vault union-testnet-10 --field cosmos-private-key --reveal)"'';
          gas_config = {
            type = "fixed";
            gas_price = "0.003";
            gas_denom = "ubbn";
            gas_multiplier = "1.1";
            max_gas = 10000000;
          };
          apps = {
            ucs03 = ucs03-configs.cw20 cw20-base;
          };
          bech32_prefix = "bbn";
          lightclients = [
            "cometbls"
            "tendermint"
            "trusted-mpt"
            "state-lens-ics23-mpt"
          ];
        }
        {
          chain-id = "stride-internal-1";
          name = "stride-testnet";
          rpc_url = "https://stride-testnet-rpc.polkachu.com";
          private_key = ''"$1"'';
          gas_config = {
            type = "fixed";
            gas_price = "0.1";
            gas_denom = "ustrd";
            gas_multiplier = "1.1";
            max_gas = 60000000;
          };
          apps = {
            ucs03 = (ucs03-configs.cw20 cw20-base) // {
              rate_limit_disabled = true;
            };
          };
          permissioned = true;
          bech32_prefix = "stride";
          lightclients = [
            "cometbls"
            "tendermint"
            "state-lens-ics23-mpt"
            # "state-lens-ics23-smt"
          ];
        }
        {
          chain-id = "xion-testnet-2";
          ucs04-chain-id = "xion.xion-testnet-2";
          name = "xion-testnet";
          rpc_url = "https://rpc.xion-testnet-2.burnt.com/";
          private_key = ''"$(op item get deployer --vault union-testnet-10 --field cosmos-private-key --reveal)"'';
          gas_config = {
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
          bech32_prefix = "xion";
          lightclients = [
            "cometbls"
            "tendermint"
            "state-lens-ics23-mpt"
            "state-lens-ics23-ics23"
          ];
        }
        {
          chain-id = "mantra-dukong-1";
          name = "mantra-testnet";
          rpc_url = "https://rpc.dukong.mantrachain.io/";
          private_key = ''"$1"'';
          gas_config = {
            type = "fixed";
            gas_price = "0.015";
            gas_denom = "uom";
            gas_multiplier = "1.4";
            max_gas = 60000000;
          };
          apps = {
            ucs03 = (ucs03-configs.cw20 cw20-base) // {
              rate_limit_disabled = true;
            };
          };
          bech32_prefix = "mantra";
          lightclients = [
            "cometbls"
            "tendermint"
            "state-lens-ics23-mpt"
          ];
        }
      ];

      # directory => {}
      all-lightclients = [
        {
          name = "base";
          dir = "base";
          client-type = "base";
        }
        {
          name = "bob";
          dir = "bob";
          client-type = "bob";
        }
        {
          name = "arbitrum";
          dir = "arbitrum";
          client-type = "arbitrum";
        }
        {
          name = "berachain";
          dir = "berachain";
          client-type = "berachain";
        }
        {
          name = "cometbls";
          dir = "cometbls";
          client-type = "cometbls";
        }
        {
          name = "ethereum";
          dir = "ethereum";
          client-type = "ethereum";
        }
        {
          name = "trusted-mpt";
          dir = "trusted-mpt";
          client-type = "trusted/evm/mpt";
        }
        {
          name = "ethermint";
          dir = "ethermint";
          client-type = "ethermint";
        }
        {
          name = "tendermint";
          dir = "tendermint";
          client-type = "tendermint";
        }
        {
          name = "tendermint-bls";
          dir = "tendermint";
          client-type = "tendermint";
          features = [ "bls" ];
        }
        # {
        #   name = "movement";
        #   dir = "movement";
        #   client-type = "movement";
        # }
        {
          name = "state-lens-ics23-mpt";
          dir = "state-lens-ics23-mpt";
          client-type = "state-lens/ics23/mpt";
        }
        # {
        #   name = "state-lens-ics23-smt";
        #   dir = "state-lens-ics23-smt";
        #   client-type = "state-lens/ics23/smt";
        # }
        {
          name = "state-lens-ics23-ics23";
          dir = "state-lens-ics23-ics23";
          client-type = "state-lens/ics23/ics23";
        }
        {
          name = "sui";
          dir = "sui";
          client-type = "sui";
        }
        {
          name = "parlia";
          dir = "parlia";
          client-type = "parlia";
        }
      ];

      # client type => package name
      all-apps = {
        # ucs00-pingpong = {
        #   name = "ucs00";
        # };
        ucs03-zkgm = {
          name = "ucs03";
        };
      };

      ucs03-configs = {
        cw20 = cw20-impl: {
          type = "cw20";
          path = "${ucs03-zkgm.release}";
          cw_account_path = "${cw-account.release}";
          token_minter_path = "${cw20-token-minter.release}";
          token_minter_config = {
            cw20 = {
              cw20_impl = "${cw20-impl.release}";
            };
          };
          rate_limit_disabled = false;
        };
        osmosis_tokenfactory = {
          type = "osmosis-tokenfactory";
          rate_limit_disabled = false;
          path = "${ucs03-zkgm.release}";
          cw_account_path = "${cw-account.release}";
          token_minter_path = "${osmosis-tokenfactory-token-minter.release}";
          token_minter_config = {
            osmosis_tokenfactory = { };
          };
        };
      };

      get-git-rev =
        {
          rpc_url,
          ...
        }:
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
              '${rpc_url}/abci_query?path=%22/cosmwasm.wasm.v1.Query/Code%22&data=0x'"$(
                buf \
                  convert \
                  ${cosmwasmProtoDefs}/cosmwasm.proto \
                  --type=cosmwasm.QueryCodeRequest \
                  --from=<(
                    echo "{\"code_id\":$(
                      curl -L \
                        --silent \
                        '${rpc_url}/abci_query?path=%22/cosmwasm.wasm.v1.Query/ContractInfo%22&data=0x'"$(
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
        };

      deploy =
        args@{
          name,
          rpc_url,
          gas_config,
          private_key,
          permissioned ? false,
          ...
        }:
        pkgs.writeShellApplication {
          name = "${name}-deploy-full";
          runtimeInputs = [ cosmwasm-deployer ];
          text = ''
            PRIVATE_KEY=${private_key} \
            RUST_LOG=info \
              cosmwasm-deployer \
              deploy-full \
              --contracts ${chain-contracts-config-json args} \
              ${if permissioned then "--permissioned " else ""} \
              --rpc-url ${rpc_url} \
              ${mk-gas-args gas_config}
          '';
        };

      whitelist-relayers =
        {
          name,
          ucs04-chain-id,
          rpc_url,
          gas_config,
          private_key,
          ...
        }:
        pkgs.writeShellApplication {
          name = "${name}-whitelist-relayers";
          runtimeInputs = [ cosmwasm-deployer ];
          text = ''
            PRIVATE_KEY=${private_key} \
            RUST_LOG=info \
              cosmwasm-deployer \
              tx \
              whitelist-relayers \
              --rpc-url ${rpc_url} \
              --contract ${(getDeployment ucs04-chain-id).core.address} \
              ${mk-gas-args gas_config} "$@"
          '';
        };

      migrate-contract =
        {
          name,
          rpc_url,
          gas_config,
          private_key,
          ...
        }:
        pkgs.writeShellApplication {
          name = "${name}-migrate-contract";
          runtimeInputs = [ cosmwasm-deployer ];
          text = ''
            PRIVATE_KEY=${private_key} \
            RUST_LOG=info \
              cosmwasm-deployer \
              migrate \
              --rpc-url ${rpc_url} \
              --address "''${1:?address must be set (first argument to this script))}" \
              --new-bytecode "''${2:?new bytecode path must be set (second argument to this script))}" \
              ${mk-gas-args gas_config} \
              "''${@:3}"
          '';
        };

      # migrate the admin to the multisig address
      finalize-deployment =
        {
          name,
          rpc_url,
          gas_config,
          private_key,
          multisig_address,
          bech32_prefix,
          ...
        }:
        pkgs.writeShellApplication {
          name = "${name}-deploy-full";
          runtimeInputs = [
            ibc-union-contract-addresses
            cosmwasm-deployer
          ];
          text = ''
            DEPLOYER=$(
              PRIVATE_KEY=${private_key} \
                cosmwasm-deployer \
                address-of-private-key \
                --bech32-prefix ${bech32_prefix}
            )
            ADDRESSES=$(ibc-union-contract-addresses "$DEPLOYER")

            echo "$DEPLOYER"
            echo "$ADDRESSES"

            PRIVATE_KEY=${private_key} \
            RUST_LOG=info \
              cosmwasm-deployer \
              migrate-admin \
              --new-admin ${multisig_address} \
              --addresses <(echo "$ADDRESSES") \
              --rpc-url ${rpc_url} \
              ${mk-gas-args gas_config}
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
            lightclient = builtins.listToAttrs (
              map (
                { name, client-type, ... }:
                {
                  name = client-type;
                  value = (mk-lightclient name).release;
                }
              ) (builtins.filter ({ name, ... }: builtins.elem name lightclients) all-lightclients)
            );
            app = apps;
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
          private_key,
          rpc_url,
          bech32_prefix,
          gas_config,
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
                  ibc-union-contract-addresses
                  cosmwasm-deployer
                ];
                text = ''
                  DEPLOYER=$(
                    PRIVATE_KEY=${private_key} \
                      cosmwasm-deployer \
                      address-of-private-key \
                      --bech32-prefix ${bech32_prefix}
                  )
                  echo "deployer address: $DEPLOYER"
                  ADDRESSES=$(ibc-union-contract-addresses "$DEPLOYER")

                  PRIVATE_KEY=${private_key} \
                  RUST_LOG=info \
                    cosmwasm-deployer \
                    migrate \
                    --rpc-url ${rpc_url} \
                    --address "$(echo "$ADDRESSES" | jq '.lightclient."${
                      (get-lightclient (l: l.name == lc)).client-type
                    }"' -r)" \
                    --new-bytecode ${(mk-lightclient lc).release} \
                    ${mk-gas-args gas_config} \
                    "$@"
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
              full-app = pkgs.lib.lists.findFirst (a: a.value.name == app) (throw "???") (
                pkgs.lib.attrsets.mapAttrsToList pkgs.lib.attrsets.nameValuePair all-apps
              );
            in
            {
              inherit name;
              value = pkgs.writeShellApplication {
                name = "${args.name}-${name}";
                runtimeInputs = [
                  ibc-union-contract-addresses
                  cosmwasm-deployer
                  pkgs.jq
                ];
                text = ''
                  PRIVATE_KEY=${private_key} \
                  RUST_LOG=info \
                    cosmwasm-deployer \
                    store-code \
                    --rpc-url ${rpc_url} \
                    --bytecode ${apps.ucs03.token_minter_path} \
                    --output token-minter-code-id.txt \
                    ${mk-gas-args gas_config}

                  echo "token minter code id: $(cat token-minter-code-id.txt)"

                  PRIVATE_KEY=${private_key} \
                  RUST_LOG=info \
                    cosmwasm-deployer \
                    store-code \
                    --rpc-url ${rpc_url} \
                    --bytecode ${apps.ucs03.token_minter_config.cw20.cw20_impl} \
                    --output cw20-impl-code-id.txt \
                    ${mk-gas-args gas_config}

                  echo "cw20 impl code id: $(cat cw20-impl-code-id.txt)"

                  PRIVATE_KEY=${private_key} \
                  RUST_LOG=info \
                    cosmwasm-deployer \
                    store-code \
                    --rpc-url ${rpc_url} \
                    --bytecode ${apps.ucs03.cw_account_path} \
                    --output cw-account-code-id.txt \
                    ${mk-gas-args gas_config}

                  echo "cw-account code id: $(cat cw-account-code-id.txt)"

                  PRIVATE_KEY=${private_key} \
                  RUST_LOG=info \
                    cosmwasm-deployer \
                    proxy-code-id \
                    --rpc-url ${rpc_url} \
                    --output proxy-code-id.txt \
                    ${mk-gas-args gas_config}

                  echo "proxy code id: $(cat proxy-code-id.txt)"

                  DEPLOYER=$(
                    PRIVATE_KEY=${private_key} \
                      cosmwasm-deployer \
                      address-of-private-key \
                      --bech32-prefix ${bech32_prefix}
                  )

                  echo "deployer address: $DEPLOYER"

                  ADDRESSES=$(ibc-union-contract-addresses "$DEPLOYER")

                  PRIVATE_KEY=${private_key} \
                  RUST_LOG=info \
                    cosmwasm-deployer \
                    migrate \
                    --rpc-url ${rpc_url} \
                    --address "$(echo "$ADDRESSES" | jq '.app."${app}"' -r)" \
                    --message "{\"token_minter_migration\":{\"new_code_id\":$(cat token-minter-code-id.txt),\"msg\":\"$(echo "{\"new_cw20_code_id\":$(cat cw20-impl-code-id.txt)}" | base64)\"}, \"rate_limit_disabled\":${
                      if apps.ucs03.rate_limit_disabled then "true" else "false"
                    }, \"cw_account_code_id\": $(cat cw-account-code-id.txt), \"dummy_code_id\": $(cat proxy-code-id.txt)}" \
                    --force \
                    --new-bytecode ${(mk-app full-app.name).release} \
                    ${mk-gas-args gas_config}

                  rm token-minter-code-id.txt
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
                ibc-union-contract-addresses
                cosmwasm-deployer
              ];
              text = ''
                DEPLOYER=$(
                  PRIVATE_KEY=${private_key} \
                    cosmwasm-deployer \
                    address-of-private-key \
                    --bech32-prefix ${bech32_prefix}
                )
                echo "deployer address: $DEPLOYER"
                ADDRESSES=$(ibc-union-contract-addresses "$DEPLOYER")

                PRIVATE_KEY=${private_key} \
                RUST_LOG=info \
                  cosmwasm-deployer \
                  migrate \
                  --rpc-url ${rpc_url} \
                  --address "$(echo "$ADDRESSES" | jq '.core' -r)" \
                  --new-bytecode ${ibc-union.release} \
                  ${mk-gas-args gas_config}
              '';
            };
          }
        );

      ibc-union-contract-addresses = pkgs.writeShellApplication {
        name = "ibc-union-contract-addresses";
        runtimeInputs = [ cosmwasm-deployer ];
        text = ''
          cosmwasm-deployer \
            addresses \
            ${
              pkgs.lib.strings.concatStrings (map (lc: " --lightclient ${lc.client-type}") all-lightclients)
            } \
            ${
              pkgs.lib.strings.concatStrings (map (a: " --${all-apps.${a}.name}") (builtins.attrNames all-apps))
            } \
            --deployer "''${1:?deployer must be set (first argument to this script))}" ''${2+--output $2} 
        '';
      };

      get-lightclient =
        f:
        pkgs.lib.lists.findSingle f (throw "lightclient not found")
          (throw "many matching lightclients found")
          all-lightclients;

      mk-lightclient =
        name:
        let
          lc = get-lightclient (lc: lc.name == name);
        in
        (lc.hook or (d: d)) (
          crane.buildWasmContract "cosmwasm/ibc-union/lightclient/${lc.dir}" {
            features = lc.features or null;
          }
        );

      mk-app = dir: crane.buildWasmContract "cosmwasm/ibc-union/app/${dir}" { };

      # ucs00-pingpong = crane.buildWasmContract {
      #   crateDirFromRoot = "cosmwasm/ucs00-pingpong";
      # };

      ucs03-zkgm = crane.buildWasmContract "cosmwasm/ibc-union/app/ucs03-zkgm" { };

      cw-account = crane.buildWasmContract "cosmwasm/cw-account" { };

      cw20-base = crane.buildWasmContract "cosmwasm/cw20-base" { };

      cw20-wrapped-tokenfactory = crane.buildWasmContract "cosmwasm/cw20-wrapped-tokenfactory" { };

      ibc-union = crane.buildWasmContract "cosmwasm/ibc-union/core" { };

      multicall = crane.buildWasmContract "cosmwasm/multicall" { };

      # native-token-minter = crane.buildWasmContract {
      #   crateDirFromRoot = "cosmwasm/native-token-minter";
      # };

      cw20-token-minter = crane.buildWasmContract "cosmwasm/cw20-token-minter" { };

      osmosis-tokenfactory-token-minter =
        crane.buildWasmContract "cosmwasm/osmosis-tokenfactory-token-minter"
          { };

      update-deployments-json =
        {
          name,
          rpc_url,
          ucs04-chain-id,
          lightclients,
          apps,
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
              --rpc-url ${rpc_url} \
              ${pkgs.lib.concatMapStringsSep " " (lc: "--lightclient ${lc}") lightclients} \
              ${if apps ? ucs03 then "--ucs03 --ucs03-minter ${apps.ucs03.type}" else ""} \
              ${if apps ? ucs00 then "--ucs00" else ""}
          '';
        };
    in
    {
      packages =
        {
          inherit
            bytecode-base
            ucs03-zkgm
            cw20-base
            cw20-wrapped-tokenfactory
            cosmwasm-deployer
            cw20-token-minter
            osmosis-tokenfactory-token-minter
            ibc-union
            multicall
            cw-account
            ;
          cosmwasm-scripts =
            {
              inherit ibc-union-contract-addresses;
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
                      update-deployments-json = update-deployments-json chain;
                      finalize-deployment = finalize-deployment chain;
                      get-git-rev = get-git-rev chain;
                      whitelist-relayers = whitelist-relayers chain;
                      migrate-contract = migrate-contract chain;
                    }
                    // (chain-migration-scripts chain)
                  );
                }) networks
              )
            ));
        }
        //
          # all light clients
          (builtins.listToAttrs (
            map (
              { name, ... }:
              let
                lc = mk-lightclient name;
              in
              {
                name = lc.passthru.packageName;
                value = lc;
              }
            ) all-lightclients
          ))
        //
          # all apps
          (pkgs.lib.mapAttrs' (n: _v: rec {
            name = value.passthru.packageName;
            value = mk-app n;
          }) all-apps);
    };
}
