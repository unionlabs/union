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
        let
          json = builtins.fromJSON (builtins.readFile ../deployments/deployments.json);
        in
        chainId:
        (pkgs.lib.lists.findSingle (deployment: deployment.chain_id == chainId)
          (throw "deployment for ${chainId} not found")
          (throw "many deployments for ${chainId} found")
          json
        ).deployments;

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
            ucs03 = ucs03-configs.cw20 // {
              rate_limit_disabled = true;
            };
          };
          # lightclients = pkgs.lib.lists.remove "cometbls" (builtins.attrNames all-lightclients);
          lightclients = [
            # "tendermint-bls"
            "berachain"
            # "ethereum"
            # "trusted-mpt"
            # "bob"
          ];
        }
        {
          chain-id = "union-testnet-10";
          name = "union-testnet-10";
          rpc_url = "https://rpc.rpc-node.union-testnet-10.union.build";
          private_key = ''"$(op item get deployer --vault union-testnet-10 --field cosmos-private-key --reveal)"'';
          gas_config = {
            type = "feemarket";
            gas_multiplier = 1.4;
          };
          apps = {
            ucs03 = ucs03-configs.cw20 // {
              rate_limit_disabled = true;
            };
          };
          bech32_prefix = "union";
          lightclients = [
            "arbitrum"
            "bob"
            "berachain"
            "ethereum"
            "trusted-mpt"
            # "ethermint"
            "tendermint-bls"
            "movement"
            "state-lens-ics23-mpt"
            "state-lens-ics23-smt"
          ];
        }
        {
          chain-id = "union-1";
          name = "union";
          rpc_url = "https://rpc.rpc-node.union-1.union.build";
          private_key = ''"$(op item get deployer --vault union-testnet-10 --field cosmos-private-key --reveal)"'';
          gas_config = {
            type = "feemarket";
            max_gas = 10000000;
            gas_multiplier = 1.4;
          };
          apps = {
            ucs03 = ucs03-configs.cw20;
          };
          bech32_prefix = "union";
          lightclients = [
            "arbitrum"
            "bob"
            "berachain"
            "ethereum"
            "trusted-mpt"
            # "ethermint"
            "tendermint-bls"
            "movement"
            "state-lens-ics23-mpt"
            "state-lens-ics23-smt"
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
            ucs03 = ucs03-configs.cw20 // {
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
          name = "osmosis-testnet";
          rpc_url = "https://osmosis-testnet-rpc.polkachu.com";
          private_key = ''"$(op item get deployer --vault union-testnet-10 --field cosmos-private-key --reveal)"'';
          gas_config = {
            type = "fixed";
            gas_price = "0.05";
            gas_denom = "uosmo";
            gas_multiplier = "1.1";
            max_gas = 10000000;
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
          chain-id = "bbn-test-5";
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
            ucs03 = ucs03-configs.cw20 // {
              rate_limit_disabled = true;
            };
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
          chain-id = "bbn-1";
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
            ucs03 = ucs03-configs.cw20;
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
            ucs03 = ucs03-configs.cw20 // {
              rate_limit_disabled = true;
            };
          };
          permissioned = true;
          bech32_prefix = "stride";
          lightclients = [
            "cometbls"
            "tendermint"
            "state-lens-ics23-mpt"
            "state-lens-ics23-smt"
          ];
        }
        {
          chain-id = "xion-testnet-2";
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
            ucs03 = ucs03-configs.cw20 // {
              rate_limit_disabled = true;
            };
          };
          bech32_prefix = "xion";
          lightclients = [
            "cometbls"
            "tendermint"
            "state-lens-ics23-mpt"
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
            ucs03 = ucs03-configs.cw20 // {
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
        # {
        #   name = "ethermint";
        #   dir = "ethermint";
        #   client-type = "ethermint";
        # }
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
        {
          name = "movement";
          dir = "movement";
          client-type = "movement";
        }
        {
          name = "state-lens-ics23-mpt";
          dir = "state-lens-ics23-mpt";
          client-type = "state-lens/ics23/mpt";
        }
        {
          name = "state-lens-ics23-smt";
          dir = "state-lens-ics23-smt";
          client-type = "state-lens/ics23/smt";
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
        cw20 = {
          path = "${ucs03-zkgm.release}";
          token_minter_path = "${cw20-token-minter.release}";
          token_minter_config = {
            cw20 = {
              cw20_base = "${cw20-base.release}";
            };
          };
          rate_limit_disabled = false;
        };
        osmosis_tokenfactory = {
          rate_limit_disabled = false;
          path = "${ucs03-zkgm.release}";
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
              '${rpc_url}/abci_query?path="/cosmwasm.wasm.v1.Query/Code"&data=0x'"$(
                buf \
                  convert \
                  ${cosmwasmProtoDefs}/cosmwasm.proto \
                  --type=cosmwasm.QueryCodeRequest \
                  --from=<(
                    echo "{\"code_id\":$(
                      curl -L \
                        --silent \
                        '${rpc_url}/abci_query?path="/cosmwasm.wasm.v1.Query/ContractInfo"&data=0x'"$(
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
              --contracts ${chain-deployments-json args} \
              ${if permissioned then "--permissioned " else ""} \
              --rpc-url ${rpc_url} \
              ${mk-gas-args gas_config}
          '';
        };

      whitelist-relayers =
        {
          name,
          chain-id,
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
              --contract ${(getDeployment chain-id).core.address} \
              ${mk-gas-args gas_config} "$@"
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

      chain-deployments-json =
        { lightclients, apps, ... }:
        pkgs.writeText "contracts.json" (
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
                    ${mk-gas-args gas_config}
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
                    --message "{\"token_minter_migration\":{\"new_code_id\":$(cat token-minter-code-id.txt),\"msg\":\"$(echo '{}' | base64)\"}, \"rate_limit_disabled\":${
                      if apps.ucs03.rate_limit_disabled then "true" else "false"
                    }}" \
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

      cw20-base = crane.buildWasmContract "cosmwasm/cw20-base" { };

      ibc-union = crane.buildWasmContract "cosmwasm/ibc-union/core" { };

      multicall = crane.buildWasmContract "cosmwasm/multicall" { };

      # native-token-minter = crane.buildWasmContract {
      #   crateDirFromRoot = "cosmwasm/native-token-minter";
      # };

      cw20-token-minter = crane.buildWasmContract "cosmwasm/cw20-token-minter" { };

      osmosis-tokenfactory-token-minter =
        crane.buildWasmContract "cosmwasm/osmosis-tokenfactory-token-minter"
          { };

      # update-deployments-json deployer
      update-deployments-json =
        {
          name,
          rpc_url,
          apps,
          ...
        }:
        pkgs.writeShellApplication {
          name = "${name}-update-deployments-json";
          runtimeInputs = [
            cosmwasm-deployer
            ibc-union-contract-addresses
            (get-git-rev { inherit rpc_url; })
            pkgs.jq
            pkgs.buf
            pkgs.xxd
            pkgs.curl
            pkgs.moreutils
          ];
          text = ''
            ${ensureAtRepositoryRoot}

            DEPLOYMENTS_FILE="deployments/deployments.json"
            export DEPLOYMENTS_FILE

            ADDRESSES=$(ibc-union-contract-addresses "$1")
            echo "addresses: $ADDRESSES"

            HEIGHTS=$(cosmwasm-deployer init-heights --rpc-url "${rpc_url}" --addresses <(echo "$ADDRESSES"))
            echo "heights: $HEIGHTS"

            echo "updating heights..."

            DEPLOYMENTS=$(echo "$ADDRESSES" | jq \
              --arg deployer "$1" \
              --argjson heights "$HEIGHTS" \
              '. as $in | {
                deployer: $deployer,
                core: {
                  address: .core,
                  height: $heights[.core]
                },
                lightclient: (reduce
                  (.lightclient | keys[]) as $key
                  ({};
                    if
                      $heights[$in.lightclient[$key]] != null
                    then
                      . + {
                        ($key): {
                          address: $in.lightclient[$key],
                          height: $heights[$in.lightclient[$key]]
                        }
                      }
                    else
                      .
                    end
                  )
                ),
                app: (reduce
                  (.app | keys[]) as $key
                  ({};
                    if
                      $heights[$in.app[$key]] != null
                    then
                      . + {
                        ($key): {
                          address: $in.app[$key],
                          height: $heights[$in.app[$key]]
                        }
                      }
                    else
                      .
                    end
                  )
                ),
              }')

            echo "deployments: $DEPLOYMENTS"

            echo "updating commits..."

            DEPLOYMENTS=$(
              echo "$DEPLOYMENTS" \
                | jq '.core.commit = $commit' \
                  --arg commit "$(get-git-rev "$(echo "$ADDRESSES" | jq .core -r)")"
            )

            for key in lightclient app ; do
              echo "key: $key"
                while read -r subkey ; do
                  echo "$key: $subkey"
                  DEPLOYMENTS=$(
                    echo "$DEPLOYMENTS" \
                      | jq '.[$key][$subkey].commit = $commit' \
                        --arg subkey "$subkey" \
                        --arg key "$key" \
                        --arg commit "$(
                          get-git-rev "$(
                            echo "$ADDRESSES" \
                              | jq -r '.[$key][$subkey]' \
                                --arg subkey "$subkey" \
                                --arg key "$key"
                          )"
                        )"
                  )
                done <<< "$(echo "$DEPLOYMENTS" \
                  | jq -r '.[$key] | keys[]' \
                    --arg key "$key")"
            done

            # get the ucs03 minter address and info
            if [ "$(echo "$ADDRESSES" | jq '.app | has("ucs03")')" == "true" ]; then
              MINTER_ADDRESS="$(
                curl -L \
                  --silent \
                  '${rpc_url}/abci_query?path="/cosmwasm.wasm.v1.Query/SmartContractState"&data=0x'"$(
                    buf \
                      convert \
                      ${cosmwasmProtoDefs}/cosmwasm.proto \
                      --type=cosmwasm.QuerySmartContractStateRequest \
                      --from=<(
                        echo \
                          "{\"address\":\"$(
                            echo "$ADDRESSES" | jq -r '.app.ucs03'
                          )\",\"query_data\":\"$(
                            echo '{"get_minter":{}}' | base64 -w0
                          )\"}"
                      )#format=json \
                      | xxd -c 0 -ps
                    )" \
                    | jq .result.response.value -r \
                    | base64 -d \
                    | buf \
                      convert \
                      ${cosmwasmProtoDefs}/cosmwasm.proto \
                      --type=cosmwasm.QuerySmartContractStateResponse \
                      --from=-#format=binpb \
                    | jq .data -r \
                    | base64 -d \
                    | jq . -r
                )"

              echo "minter_address: $MINTER_ADDRESS"

              DEPLOYMENTS=$(
                echo "$DEPLOYMENTS" \
                  | jq '.app.ucs03.minter = { type: $type, address: $address, commit: $commit }' \
                    --arg type ${builtins.elemAt (builtins.attrNames apps.ucs03.token_minter_config) 0} \
                    --arg address "$MINTER_ADDRESS" \
                    --arg commit "$(get-git-rev "$MINTER_ADDRESS")"
              )
            fi

            echo "deployments: $DEPLOYMENTS"

            CHAIN_ID="$(curl -L --silent ${rpc_url}/status | jq .result.node_info.network -r)"
            export CHAIN_ID

            echo "chain id: $CHAIN_ID"

            jq \
              '. |= map(if .chain_id == $chain_id then .deployments = $deployments else . end)' \
              "$DEPLOYMENTS_FILE" \
              --arg chain_id "$CHAIN_ID" \
              --argjson deployments "$DEPLOYMENTS" \
            | sponge "$DEPLOYMENTS_FILE"
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
            cosmwasm-deployer
            cw20-token-minter
            osmosis-tokenfactory-token-minter
            ibc-union
            multicall
            ;
          cosmwasm-scripts =
            {
              inherit ibc-union-contract-addresses;
              update-deployments-json = pkgs.writeShellApplication {
                name = "update-deployments-json";
                text =
                  # TODO: Merge this script with the one in evm.nix
                  let
                    deployments = builtins.filter (deployment: deployment.ibc_interface == "ibc-cosmwasm") (
                      builtins.fromJSON (builtins.readFile ../deployments/deployments.json)
                    );
                    getNetwork =
                      chainId:
                      pkgs.lib.lists.findSingle (network: network.chain-id == chainId)
                        (throw "no network found with chain id ${chainId}")
                        (throw "many networks with chain id ${chainId} found")
                        networks;
                  in
                  pkgs.lib.concatMapStringsSep "\n\n" (entry: ''
                    echo "updating ${entry.universal_chain_id}"
                    ${
                      pkgs.lib.getExe
                        self'.packages.cosmwasm-scripts.${(getNetwork entry.chain_id).name}.update-deployments-json
                    } ${entry.deployments.deployer}
                  '') deployments;
              };
            }
            // (pkgs.mkRootDrv "cosmwasm-scripts" (
              builtins.listToAttrs (
                map (chain: {
                  inherit (chain) name;
                  value = pkgs.mkRootDrv chain.name (
                    {
                      chain-deployments-json = chain-deployments-json chain;
                      deploy = deploy chain;
                      update-deployments-json = update-deployments-json chain;
                      finalize-deployment = finalize-deployment chain;
                      get-git-rev = get-git-rev chain;
                      whitelist-relayers = whitelist-relayers chain;
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
