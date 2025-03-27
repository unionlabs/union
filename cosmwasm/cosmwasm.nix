_: {
  perSystem =
    {
      self',
      crane,
      pkgs,
      dbg,
      ...
    }:
    let
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
          static =
            {
              gas_denom,
              gas_multiplier,
              gas_price,
              max_gas,
            }:
            ''
              --gas static \
              --gas-price ${toString gas_price} \
              --gas-denom ${toString gas_denom} \
              --gas-multiplier ${toString gas_multiplier} \
              --max-gas ${toString max_gas}
            '';
          feemarket = _: ''--gas feemarket'';
        }
        .${type}
          (builtins.removeAttrs config [ "type" ]);

      networks = [
        {
          name = "union-devnet";
          rpc_url = "http://localhost:26657";
          # alice from the devnet keyring
          private_key = "0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f";
          gas_config = {
            type = "feemarket";
          };
          ucs03_type = "cw20";
          bech32_prefix = "union";
          apps = {
            ucs03 = ucs03-configs.cw20;
          };
          # lightclients = pkgs.lib.lists.remove "cometbls" (builtins.attrNames all-lightclients);
          lightclients = [
            "ethereum"
            "trusted-mpt"
          ];
        }
        {
          name = "union-testnet-9";
          rpc_url = "https://rpc.testnet-9.union.build";
          private_key = ''"$1"'';
          gas_config = {
            type = "static";
            gas_denom = "muno";
            gas_multiplier = "1.1";
            gas_price = "1.0";
            max_gas = 200000000;
          };
          apps = {
            ucs03 = ucs03-configs.cw20;
          };
          bech32_prefix = "union";
          lightclients = [
            "arbitrum"
            "berachain"
            "ethereum"
            "ethermint"
            "tendermint-bls"
            "movement"
            "state-lens-ics23-mpt"
            "state-lens-ics23-smt"
          ];
        }
        {
          name = "union-testnet-10";
          rpc_url = "https://rpc.rpc-node.union-testnet-10.union.build";
          private_key = ''"$(op item get deployer --vault union-testnet-10 --field cosmos-private-key)"'';
          gas_config = {
            type = "feemarket";
          };
          apps = {
            ucs03 = ucs03-configs.cw20;
          };
          bech32_prefix = "union";
          lightclients = [
            # "arbitrum"
            # "berachain"
            "ethereum"
            # "ethermint"
            # "tendermint-bls"
            # "movement"
            # "state-lens-ics23-mpt"
            # "state-lens-ics23-smt"
          ];
        }
        {
          name = "stargaze-testnet";
          rpc_url = "https://rpc.elgafar-1.stargaze.chain.kitchen";
          private_key = ''"$1"'';
          gas_config = {
            type = "static";
            gas_price = "1.0";
            gas_denom = "ustars";
            gas_multiplier = "1.1";
            max_gas = 10000000;
          };
          apps = {
            ucs03 = ucs03-configs.cw20;
          };
          bech32_prefix = "stars";
          lightclients = [
            "cometbls"
            "tendermint"
            "state-lens-ics23-mpt"
          ];
        }
        {
          name = "osmosis-testnet";
          rpc_url = "https://osmosis-testnet-rpc.polkachu.com";
          private_key = ''"$1"'';
          gas_config = {
            type = "static";
            gas_price = "0.05";
            gas_denom = "uosmo";
            gas_multiplier = "1.1";
            max_gas = 300000000;
          };
          apps = {
            ucs03 = ucs03-configs.cw20;
          };
          bech32_prefix = "osmo";
          lightclients = [
            "cometbls"
            "tendermint"
            "state-lens-ics23-mpt"
          ];
        }
        {
          name = "babylon-testnet";
          rpc_url = "https://babylon-testnet-rpc.polkachu.com";
          private_key = ''"$1"'';
          gas_config = {
            type = "static";
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
            "state-lens-ics23-mpt"
          ];
        }
        {
          name = "stride-testnet";
          rpc_url = "https://stride-testnet-rpc.polkachu.com";
          private_key = ''"$1"'';
          gas_config = {
            type = "static";
            gas_price = "0.1";
            gas_denom = "ustrd";
            gas_multiplier = "1.1";
            max_gas = 60000000;
          };
          apps = {
            ucs03 = ucs03-configs.cw20;
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
          name = "xion-testnet";
          rpc_url = "https://rpc.xion-testnet-2.burnt.com/";
          private_key = ''"$1"'';
          gas_config = {
            type = "static";
            gas_price = "0.002";
            gas_denom = "uxion";
            gas_multiplier = "1.5";
            max_gas = 60000000;
          };
          apps = {
            ucs03 = ucs03-configs.cw20;
          };
          bech32_prefix = "xion";
          lightclients = [
            "cometbls"
            "tendermint"
            "state-lens-ics23-mpt"
          ];
        }
        {
          name = "mantra-testnet";
          rpc_url = "https://rpc.dukong.mantrachain.io/";
          private_key = ''"$1"'';
          gas_config = {
            type = "static";
            gas_price = "0.015";
            gas_denom = "uom";
            gas_multiplier = "1.4";
            max_gas = 60000000;
          };
          apps = {
            ucs03 = ucs03-configs.cw20;
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
          path = "${self'.packages.ucs03-zkgm}";
          token_minter_path = "${self'.packages.cw20-token-minter}";
          token_minter_config = {
            cw20 = {
              cw20_base = "${cw20-base}";
            };
          };
        };
        native = {
          path = "${self'.packages.ucs03-zkgm}";
          token_minter_path = "${self'.packages.token-factory-minter}";
          token_minter_config = {
            native = { };
          };
        };
      };

      deploy-full =
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
            core = ibc-union;
            lightclient = builtins.listToAttrs (
              map (
                { name, client-type, ... }:
                {
                  name = client-type;
                  value = mk-lightclient name;
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
              name = "migrate-lightclient-${args.name}-${lc}";
            in
            {
              inherit name;
              value = pkgs.writeShellApplication {
                inherit name;
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
                    --new-bytecode ${mk-lightclient lc} \
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
              name = "migrate-app-${args.name}-${app}";
            in
            {
              inherit name;
              value = pkgs.writeShellApplication {
                inherit name;
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
                    --address "$(echo "$ADDRESSES" | jq '.app."${app}"' -r)" \
                    --new-bytecode ${
                      mk-app
                        (pkgs.lib.lists.findFirst (a: a.value.name == app) (throw "???") (
                          pkgs.lib.attrsets.mapAttrsToList pkgs.lib.attrsets.nameValuePair all-apps
                        )).name
                    } \
                    ${mk-gas-args gas_config}
                '';
              };
            }
          ) (builtins.attrNames apps)
        ))
        // (
          let
            name = "migrate-core-${args.name}";
          in
          {
            ${name} = pkgs.writeShellApplication {
              inherit name;
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
                  --new-bytecode ${ibc-union} \
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
            --deployer "$1" ''${2+--output $2} 
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

      deployments-json-entry =
        { name, rpc_url, ... }:
        pkgs.writeShellApplication {
          name = "${name}-deployments-json-entry";
          runtimeInputs = [
            cosmwasm-deployer
            pkgs.jq
            ibc-union-contract-addresses
          ];
          text = ''
            ADDRESSES=$(ibc-union-contract-addresses "$1")
            HEIGHTS=$(cosmwasm-deployer init-heights --rpc-url "${rpc_url}" --addresses <(echo "$ADDRESSES"))
            echo "$ADDRESSES" | jq \
              --argjson heights "$HEIGHTS" \
              '. as $in | {
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
              }'
          '';
        };
    in
    {
      packages =
        {
          inherit
            bytecode-base
            ucs03-zkgm
            cosmwasm-deployer
            # native-token-minter
            cw20-token-minter
            ibc-union
            multicall
            ;
          cosmwasm-scripts =
            {
              inherit ibc-union-contract-addresses;
            }
            // (
              (builtins.listToAttrs (
                map (args: {
                  name = "chain-deployments-json-${args.name}";
                  value = chain-deployments-json args;
                }) networks
              ))
              // (builtins.listToAttrs (
                map (args: {
                  name = "deploy-full-${args.name}";
                  value = deploy-full args;
                }) networks
              ))
              // (builtins.listToAttrs (
                map (args: {
                  name = "deployments-json-entry-${args.name}";
                  value = deployments-json-entry args;
                }) networks
              ))
              // (builtins.listToAttrs (
                map (args: {
                  name = "finalize-deployment-${args.name}";
                  value = finalize-deployment args;
                }) (builtins.filter (network: network ? multisig_address) networks)
              ))
            )
            // (builtins.foldl' (a: b: a // b) { } (map chain-migration-scripts networks))
            // derivation { name = "cosmwasm-scripts"; };
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
