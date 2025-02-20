{ inputs, ... }:
{
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

      cosmwasm-deployer = crane.buildWorkspaceMember {
        crateDirFromRoot = "cosmwasm/deployer";
      };

      deploy-full =
        args@{
          name,
          rpc_url,
          gas_config,
          private_key,
          ...
        }:
        pkgs.writeShellApplication {
          name = "${name}-deploy-full";
          runtimeInputs = [ cosmwasm-deployer.packages.cosmwasm-deployer ];
          text = ''
            RUST_LOG=info \
              cosmwasm-deployer \
              deploy-full \
              --private-key ${private_key} \
              --gas-price ${toString gas_config.gas_price} \
              --gas-denom ${toString gas_config.gas_denom} \
              --gas-multiplier ${toString gas_config.gas_multiplier} \
              --max-gas ${toString gas_config.max_gas} \
              --contracts ${chain-deployments-json args} \
              --rpc-url ${rpc_url}
          '';
        };

      # directory => {}
      all-lightclients = {
        arbitrum = {
          client-type = "arbitrum";
        };
        berachain = {
          client-type = "berachain";
        };
        cometbls = {
          client-type = "cometbls";
        };
        ethereum = {
          client-type = "ethereum";
        };
        ethermint = {
          client-type = "ethermint";
        };
        tendermint = {
          client-type = "tendermint";
          # remove the cosmwasm 2.1 export such that the tendermint light client can work on chains that don't need bls verification (which is the only reason the export exists on this client)
          # in order to not mess with any potential offsets in the blob, overwrite the export with a string of the same length
          hook =
            drv:
            drv.overrideAttrs (old: {
              installPhase =
                (old.installPhase or "")
                + ''
                  ${pkgs.lib.getExe pkgs.bbe} \
                    -e 's/requires_cosmwasm_2_1/AAAAAAAAAAAAAAAAAAAAA/' \
                    -e 's/requires_cosmwasm_2_0/BBBBBBBBBBBBBBBBBBBBB/' \
                    $out \
                    -o replaced.wasm

                  # can't write directly to $out for some reason
                  mv replaced.wasm $out
                '';
            });
        };
        movement = {
          client-type = "movement";
        };
        state-lens-ics23-mpt = {
          client-type = "state-lens/ics23/mpt";
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

      # client type => package name
      all-apps = {
        # ucs00-pingpong = {
        #   name = "ucs00";
        # };
        ucs03-zkgm = {
          name = "ucs03";
        };
      };

      chain-deployments-json =
        { lightclients, apps, ... }:
        pkgs.writeText "contracts.json" (
          builtins.toJSON {
            core = ibc-union;
            lightclient = pkgs.lib.mapAttrs' (n: v: {
              name = v.client-type;
              value = mk-lightclient n;
            }) (pkgs.lib.filterAttrs (n: _: builtins.elem n lightclients) all-lightclients);
            app = apps;
          }
        );

      chain-migration-scripts =
        args@{
          name,
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
              name = "${args.name}-migrate-lightclient-${lc}";
            in
            {
              inherit name;
              value = pkgs.writeShellApplication {
                inherit name;
                runtimeInputs = [
                  ibc-union-contract-addresses
                  cosmwasm-deployer.packages.cosmwasm-deployer
                ];
                text = ''
                  DEPLOYER=$(cosmwasm-deployer address-of-private-key --private-key ${private_key} --bech32-prefix ${bech32_prefix})
                  echo "deployer address: $DEPLOYER"
                  ADDRESSES=$(ibc-union-contract-addresses "$DEPLOYER")

                  RUST_LOG=info \
                    cosmwasm-deployer \
                    migrate \
                    --rpc-url ${rpc_url} \
                    --address "$(echo "$ADDRESSES" | jq '.lightclient."${all-lightclients.${lc}.client-type}"' -r)" \
                    --new-bytecode ${mk-lightclient lc} \
                    --private-key ${private_key} \
                    --gas-price ${toString gas_config.gas_price} \
                    --gas-denom ${toString gas_config.gas_denom} \
                    --gas-multiplier ${toString gas_config.gas_multiplier} \
                    --max-gas ${toString gas_config.max_gas} \
                '';
              };
            }
          ) lightclients
        ))
        // (builtins.listToAttrs (
          map (
            app:
            let
              name = "${args.name}-migrate-app-${app}";
            in
            {
              inherit name;
              value = pkgs.writeShellApplication {
                inherit name;
                runtimeInputs = [
                  ibc-union-contract-addresses
                  cosmwasm-deployer.packages.cosmwasm-deployer
                ];
                text = ''
                  DEPLOYER=$(cosmwasm-deployer address-of-private-key --private-key ${private_key} --bech32-prefix ${bech32_prefix})
                  echo "deployer address: $DEPLOYER"
                  ADDRESSES=$(ibc-union-contract-addresses "$DEPLOYER")

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
                    --private-key ${private_key} \
                    --gas-price ${toString gas_config.gas_price} \
                    --gas-denom ${toString gas_config.gas_denom} \
                    --gas-multiplier ${toString gas_config.gas_multiplier} \
                    --max-gas ${toString gas_config.max_gas} \
                '';
              };
            }
          ) (builtins.attrNames apps)
        ))
      # // (map (
      #   _a:
      #   pkgs.writeShellApplication {
      #     name = "${name}-migrate-app-{a}";
      #     runtimeInputs = [ cosmwasm-deployer.packages.cosmwasm-deployer ];
      #     text = ''
      #       cosmwasm-deployer \
      #         addresses \
      #         ${
      #           pkgs.lib.strings.concatStrings (
      #             map (l: " --lightclient ${all-lightclients.${l}.client-type}") (builtins.attrNames all-lightclients)
      #           )
      #         } \
      #         ${
      #           pkgs.lib.strings.concatStrings (map (a: " --${all-apps.${a}.name}") (builtins.attrNames all-apps))
      #         } \
      #         --deployer "$1" ''${2+--output $2}
      #     '';
      #   }
      # ) apps)
      ;

      ibc-union-contract-addresses = pkgs.writeShellApplication {
        name = "ibc-union-contract-addresses";
        runtimeInputs = [ cosmwasm-deployer.packages.cosmwasm-deployer ];
        text = ''
          cosmwasm-deployer \
            addresses \
            ${
              pkgs.lib.strings.concatStrings (
                map (l: " --lightclient ${all-lightclients.${l}.client-type}") (builtins.attrNames all-lightclients)
              )
            } \
            ${
              pkgs.lib.strings.concatStrings (map (a: " --${all-apps.${a}.name}") (builtins.attrNames all-apps))
            } \
            --deployer "$1" ''${2+--output $2} 
        '';
      };

      networks = [
        {
          name = "union-devnet";
          rpc_url = "http://localhost:26657";
          # alice from the devnet keyring
          private_key = "0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f";
          gas_config = {
            gas_denom = "muno";
            gas_multiplier = "1.1";
            gas_price = "1.0";
            max_gas = 10000000;
          };
          ucs03_type = "cw20";
          bech32_prefix = "union";
          apps = {
            ucs03 = ucs03-configs.cw20;
          };
          lightclients = pkgs.lib.lists.remove "cometbls" (builtins.attrNames all-lightclients);
        }
        {
          name = "union-testnet";
          rpc_url = "https://rpc.testnet-9.union.build";
          private_key = ''"$1"'';
          gas_config = {
            gas_denom = "muno";
            gas_multiplier = "1.1";
            gas_price = "1.0";
            max_gas = 10000000;
          };
          apps = {
            ucs03 = ucs03-configs.cw20;
          };
          bech32_prefix = "union";
          lightclients = pkgs.lib.lists.remove "cometbls" (builtins.attrNames all-lightclients);
        }
        {
          name = "stargaze-testnet";
          rpc_url = "https://rpc.elgafar-1.stargaze.chain.kitchen";
          private_key = ''"$1"'';
          gas_config = {
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
            gas_price = "0.1";
            gas_denom = "ustrd";
            gas_multiplier = "1.1";
            max_gas = 60000000;
          };
          apps = {
            ucs03 = ucs03-configs.cw20;
          };
          bech32_prefix = "stride";
          lightclients = [
            "cometbls"
            # "tendermint"
            "state-lens-ics23-mpt"
          ];
        }
        {
          name = "xion-testnet";
          rpc_url = "https://rpc.xion-testnet-2.burnt.com/";
          private_key = ''"$1"'';
          gas_config = {
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
      ];

      mk-lightclient =
        dir:
        (all-lightclients.${dir}.hook or (d: d)) (
          crane.buildWasmContract {
            crateDirFromRoot = "cosmwasm/ibc-union/lightclient/${dir}";
          }
        );

      mk-app =
        dir:
        (crane.buildWasmContract {
          crateDirFromRoot = "cosmwasm/ibc-union/app/${dir}";
        });

      # ucs00-pingpong = crane.buildWasmContract {
      #   crateDirFromRoot = "cosmwasm/ucs00-pingpong";
      # };

      cw721-base = crane.buildRemoteWasmContract {
        src = inputs.cosmwasm-nfts;
        version = inputs.cosmwasm-nfts.rev;
        package = "cw721-base@0.18.0";
        contractFileNameWithoutExt = "cw721_base";
      };

      ucs03-zkgm = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/ibc-union/app/ucs03-zkgm";
      };

      cw20-base = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/cw20-base";
      };

      cw20-multibalance = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/cw20-multibalance";
      };

      ibc-union = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/ibc-union/core";
      };

      multicall = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/multicall";
      };

      # native-token-minter = crane.buildWasmContract {
      #   crateDirFromRoot = "cosmwasm/native-token-minter";
      # };

      cw20-token-minter = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/cw20-token-minter";
      };

      deployments-json-entry =
        { name, rpc_url, ... }:
        pkgs.writeShellApplication {
          name = "${name}-deployments-json-entry";
          runtimeInputs = [
            cosmwasm-deployer.packages.cosmwasm-deployer
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
            cw721-base
            ucs03-zkgm
            # native-token-minter
            cw20-token-minter
            ibc-union
            cw20-multibalance
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
            )
            // (builtins.foldl' (a: b: a // b) { } (map chain-migration-scripts networks))
            # // (dbg (chain-migration-scripts (builtins.elemAt networks 0)))
            # // (dbg (chain-migration-scripts (builtins.elemAt networks 1)))
            // derivation { name = "cosmwasm-scripts"; };
        }
        // cosmwasm-deployer.packages
        //
          # all light clients
          (pkgs.lib.mapAttrs' (n: _v: rec {
            name = value.passthru.packageName;
            value = mk-lightclient n;
          }) all-lightclients)
        //
          # all apps
          (pkgs.lib.mapAttrs' (n: _v: rec {
            name = value.passthru.packageName;
            value = mk-app n;
          }) all-apps);
    };
}
