{ pkgs
, dbg
, ...
}:
{ node
, chainId
, chainName
, denom
, keyType
, validatorCount
, portIncrease
, genesisOverwrites ? { }
, lightClients ? [ ]
, cosmwasmContracts ? [ ]
, startCommandOverwrite ? null
, extraPackages ? [ ]
, sdkVersion ? 50
}:
assert (builtins.isString chainId);
assert (builtins.isString chainName);
assert (builtins.isString denom);
assert (builtins.isString keyType);
assert (builtins.isInt portIncrease);
assert (builtins.isInt validatorCount);
assert (pkgs.lib.assertOneOf
  "sdkVersion"
  sdkVersion
  [ 47 50 ]);
let
  devKeyMnemonics = {
    alice = "wine parrot nominee girl exchange element pudding grow area twenty next junior come render shadow evidence sentence start rough debate feed all limb real";
    bob = "gun more barrel helmet velvet people alter depth bargain use isolate pear before frown already limb sweet response legal invest stand barrel stone conduct";
    charlie = "young soup enroll tornado mercy athlete tray resist limit spare address license cargo quantum panda useful clog autumn shoot observe input next across movie";
    dave = "allow where void replace vocal cabbage can expose rival danger stomach noodle faculty cart surround cash rice kite audit slight ten bicycle dance middle";
    erin = "hard educate knock ketchup salon obey debate one other impose smoke spoon pull describe cactus talk other merit joy great critic canvas scene lounge";
    frank = "over floor explain will stereo camera subway park pilot trick good exchange foot violin shop kite educate bracket shoulder fancy denial ill era battle";
  };

  nodeBin = pkgs.lib.getExe node;

  mkNodeMnemonic = idx:
    assert (builtins.isInt idx);
    pkgs.runCommand
      "${chainName}-mnemonic_${toString idx}"
      { buildInputs = [ pkgs.devnet-utils ]; }
      ''
        devnet-utils keygen mnemonic $(echo ${toString idx} | sha256sum - | cut -d' ' -f1) > $out

        echo "validator ${toString idx} mnemonic: $(cat $out)"
      '';

  mkNodeKey = idx:
    assert (builtins.isInt idx);
    pkgs.runCommand
      "${chainName}-node-key_${toString idx}"
      { buildInputs = [ pkgs.devnet-utils ]; }
      ''
        NODE_KEY=$(devnet-utils keygen key --key-type ed25519 "$(cat ${mkNodeMnemonic idx})" | tr -d '\n')

        echo "validator ${toString idx} node_key: $NODE_KEY"

        echo "{\"priv_key\":{\"type\":\"tendermint/PrivKeyEd25519\",\"value\":\"$NODE_KEY\"}}" > $out
      '';

  mkNodeId = idx:
    assert (builtins.isInt idx);
    pkgs.runCommand
      "${chainId}-node-id_${toString idx}"
      { buildInputs = [ ]; }
      ''
        export HOME=$(pwd)

        cp -r --no-preserve=mode ${initHome idx}/* .

        cp ${mkNodeKey idx} ./config/node_key.json
        ${nodeBin} tendermint show-node-id --home . | tr -d '\n' > $out
      '';

  mkPrivValidatorKey = idx:
    assert (builtins.isInt idx);
    pkgs.runCommand
      "${chainName}-priv-validator-key_${toString idx}"
      { buildInputs = [ ]; }
      ''
        export HOME=$(pwd)

        cp -r --no-preserve=mode ${initHome idx}/* .

        mv ./config/priv_validator_key.json $out
        echo "created valkey-${toString idx}: $(cat $out)"
      '';

  mkValGentx = idx:
    assert (builtins.isInt idx);
    pkgs.runCommand
      "${chainName}-valgentx_${toString idx}"
      {
        buildInputs = [ pkgs.jq ];
        src = addAllKeysToKeyringAndGenesis (initHome idx);
      }
      ''
        export HOME=$(pwd)

        PUBKEY="{\"@type\":\"/cosmos.crypto.${keyType}.PubKey\",\"key\":$(cat ${mkPrivValidatorKey idx} | jq ."pub_key"."value")}"

        echo "validator pubkey: $PUBKEY"

        # gentx was moved to a subcommand of genesis in sdk v50
        ${nodeBin} \
          ${if sdkVersion >= 50 then "genesis" else ""} gentx \
          valoper-${toString idx} \
          1000000000000000000000${denom} \
          --chain-id ${chainId} \
          --home $src \
          --keyring-backend test \
          --ip "0.0.0.0" \
          --pubkey "$PUBKEY" \
          --moniker validator-${toString idx} \
          --output-document $out
      '';

  initHome = idx: pkgs.runCommand
    "${chainName}-genesis-home"
    { buildInputs = [ ]; }
    ''
      export HOME=$(pwd)
      mkdir -p $out

      cat ${mkNodeMnemonic (if idx == null then 0 else idx)} | ${nodeBin} \
        init \
        testnet ${pkgs.lib.optionalString (sdkVersion >= 50) ''--default-denom ${denom}''} \
        --chain-id ${chainId} \
        --home $out \
        --recover 2>/dev/null

      ${pkgs.lib.optionalString (sdkVersion < 50) ''
        sed -i 's/: "stake"/: "${denom}"/g' $out/config/genesis.json
      ''} 2>/dev/null
    '';

  addDevKeyToKeyringAndGenesis = name: mnemonic: home:
    pkgs.runCommand
      "${chainName}-add-dev-key-${name}"
      { buildInputs = [ pkgs.jq pkgs.moreutils ]; }
      ''
        export HOME=$(pwd)
        mkdir -p $out
        cp --no-preserve=mode -r ${home}/* $out

        ls -al $out

        # Add the dev account
        echo ${mnemonic} | ${nodeBin} keys add \
          --recover ${name} \
          --keyring-backend test \
          --home $out

        # add-genesis-account was moved to a subcommand of genesis in sdk v50
        ${nodeBin} \
          ${if sdkVersion >= 50 then "genesis" else ""} add-genesis-account \
          ${name} \
          10000000000000000000000000${denom} \
          --keyring-backend test \
          --home $out
      '';

  addValoperKeyToKeyringAndGenesis = idx: home:
    assert (builtins.isInt idx);
    pkgs.runCommand
      "${chainName}-valkey_${toString idx}"
      { buildInputs = [ ]; }
      ''
        export HOME=$(pwd)
        mkdir -p $out
        cp --no-preserve=mode -r ${home}/* $out

        cat ${mkNodeMnemonic idx} | ${nodeBin} \
          keys \
          add \
          --recover valoper-${toString idx} \
          --keyring-backend test \
          --home $out

        # add-genesis-account was moved to a subcommand of genesis in sdk v50
        ${nodeBin} \
          ${if sdkVersion >= 50 then "genesis" else ""} add-genesis-account \
          valoper-${toString idx} \
          10000000000000000000000000${denom} \
          --keyring-backend test \
          --home $out
      '';

  addAllKeysToKeyringAndGenesis = home:
    pkgs.lib.foldl
      (home: f: f home)
      home
      (
        pkgs.lib.flatten [
          (pkgs.lib.mapAttrsToList addDevKeyToKeyringAndGenesis devKeyMnemonics)
          (builtins.genList addValoperKeyToKeyringAndGenesis validatorCount)
        ]
      );

  applyGenesisOverwrites = home:
    let
      overwrites = builtins.toFile "overwrite.json" (builtins.toJSON genesisOverwrites);
    in
    pkgs.runCommand "${chainName}-apply-genesis-overwrites"
      { buildInputs = [ pkgs.jq ]; }
      ''
        mkdir -p $out
        cp --no-preserve=mode -r ${home}/* $out
        jq -s '.[0] * .[1]' ${home}/config/genesis.json ${overwrites} > merge.json
        mv merge.json $out/config/genesis.json
      '';

  # calculateCw20Ics20ContractAddress = home: pkgs.runCommand "calculate-ucs01-relay-contract-address"
  #   {
  #     buildInputs = [ pkgs.jq ];
  #   }
  #   ''
  #     export HOME=$(pwd)
  #     mkdir -p $out
  #     cp --no-preserve=mode -r ${home}/* $out

  #     ALICE_ADDRESS=$(${nodeBin} keys list \
  #       --keyring-backend test \
  #       --home $out \
  #       --output json \
  #       | jq '.[] | select(.name == "alice").address' --raw-output)

  #     CODE_HASH=$(sha256sum ${self'.packages.ucs01-relay}/lib/ucs01_relay.wasm | cut -f1 -d" ")

  #     ${nodeBin} query wasm build-address $CODE_HASH $ALICE_ADDRESS ${cw-instantiate2-salt} --home $out > $out/CW20_ICS20_CONTRACT_ADDRESS
  #   '';

  # calculatePingPongAddress = home: pkgs.runCommand "calculate-ping-pong-contract-address"
  #   {
  #     buildInputs = [ pkgs.jq ];
  #   }
  #   ''
  #     export HOME=$(pwd)
  #     mkdir -p $out
  #     cp --no-preserve=mode -r ${home}/* $out

  #     ALICE_ADDRESS=$(${nodeBin} keys list \
  #       --keyring-backend test \
  #       --home $out \
  #       --output json \
  #       | jq '.[] | select(.name == "alice").address' --raw-output)

  #     CODE_HASH=$(sha256sum ${self'.packages.ucs00-pingpong}/lib/ucs00_pingpong.wasm | cut -f1 -d" ")

  #     ${nodeBin} query wasm build-address $CODE_HASH $ALICE_ADDRESS ${cw-instantiate2-salt} --home $out > $out/PING_PONG_CONTRACT_ADDRESS
  #   '';

  # addIbcConnectionToGenesis = home: pkgs.runCommand "add-ibc-connection-to-genesis"
  #   {
  #     buildInputs = [ pkgs.jq pkgs.moreutils ];
  #   }
  #   ''
  #     export HOME=$(pwd)
  #     mkdir -p $out
  #     cp --no-preserve=mode -r ${home}/* $out

  #     jq \
  #      '.app_state.ibc.connection_genesis.connections += [{
  #         "id": "connection-0",
  #         "client_id": "08-wasm-0",
  #         "versions": [{
  #           "identifier": "1",
  #           "features": [
  #             "ORDER_ORDERED", "ORDER_UNORDERED"
  #           ]
  #          }],
  #         "state": 3,
  #         "counterparty": {
  #           "client_id": "cometbls-new-0",
  #           "connection_id": "connection-0",
  #           "prefix": {
  #             "key_prefix": "aWJj"
  #           }
  #         },
  #         "delay_period": 0
  #       }]' \
  #       $out/config/genesis.json | sponge $out/config/genesis.json

  #     jq \
  #       '.app_state.ibc.connection_genesis.client_connection_paths += [{
  #           "client_id": "08-wasm-0",
  #           "paths": ["connection-0"]
  #       }]' \
  #       $out/config/genesis.json | sponge $out/config/genesis.json

  #     # Connection id sequence is advanced to prevent overlapping.
  #     jq \
  #       '.app_state.ibc.connection_genesis.next_connection_sequence = "1"' \
  #       $out/config/genesis.json | sponge $out/config/genesis.json
  #   '';

  enableAllClients = home:
    pkgs.runCommand
      "${chainName}-enable-all-clients"
      { buildInputs = [ pkgs.jq pkgs.moreutils ]; }
      ''
        export HOME=$(pwd)
        mkdir -p $out
        cp --no-preserve=mode -r ${home}/* $out

        jq \
         '.app_state.ibc.client_genesis.params.allowed_clients = [
            "06-solomachine",
            "07-tendermint",
            "08-wasm",
            "09-localhost"
          ]' \
          $out/config/genesis.json | sponge $out/config/genesis.json
      '';


  # addIbcChannelToGenesis = home: pkgs.runCommand "add-ibc-channel-to-genesis"
  #   {
  #     buildInputs = [ pkgs.jq pkgs.moreutils ];
  #   }
  #   ''
  #     export HOME=$(pwd)
  #     mkdir -p $out
  #     cp --no-preserve=mode -r ${home}/* $out

  #     ALICE_ADDRESS=$(${nodeBin} keys list \
  #       --keyring-backend test \
  #       --home $out \
  #       --output json \
  #       | jq '.[] | select(.name == "alice").address' --raw-output)

  #     CW20_ADDRESS=$(cat ${calculateCw20Ics20ContractAddress home}/CW20_ICS20_CONTRACT_ADDRESS)
  #     CW20_PORT=wasm.$CW20_ADDRESS

  #     PING_PONG_ADDRESS=$(cat ${calculatePingPongAddress home}/PING_PONG_CONTRACT_ADDRESS)
  #     PING_PONG_PORT=wasm.$PING_PONG_ADDRESS

  #     # TODO(aeryz): get the port id and channel info as parameters
  #     jq \
  #      --arg cw20_port $CW20_PORT \
  #      '.app_state.ibc.channel_genesis.channels += [{
  #         "state": 3,
  #         "ordering": 1,
  #         "counterparty": {
  #           "port_id": "transfer",
  #           "channel_id": "channel-0"
  #         },
  #         "connection_hops": ["connection-0"],
  #         "version": "ics20-1",
  #         "port_id": $cw20_port,
  #         "channel_id": "channel-0"
  #       }]' \
  #       $out/config/genesis.json | sponge $out/config/genesis.json

  #     jq \
  #      --arg cw20_port $CW20_PORT \
  #      --arg ping_pong_port $PING_PONG_PORT \
  #      '.app_state.ibc.channel_genesis.send_sequences += [{
  #         "port_id": $cw20_port,
  #         "channel_id": "channel-0",
  #         "sequence": "1"
  #       }, {
  #         "port_id": $ping_pong_port,
  #         "channel_id": "channel-1",
  #         "sequence": "1"
  #       }]' \
  #       $out/config/genesis.json | sponge $out/config/genesis.json

  #     jq \
  #      --arg cw20_port $CW20_PORT \
  #      --arg ping_pong_port $PING_PONG_PORT \
  #      '.app_state.ibc.channel_genesis.recv_sequences += [{
  #         "port_id": $cw20_port,
  #         "channel_id": "channel-0",
  #         "sequence": "1"
  #       }, {
  #         "port_id": $ping_pong_port,
  #         "channel_id": "channel-1",
  #         "sequence": "1"
  #       }]' \
  #       $out/config/genesis.json | sponge $out/config/genesis.json

  #     jq \
  #      --arg cw20_port $CW20_PORT \
  #      --arg ping_pong_port $PING_PONG_PORT \
  #      '.app_state.ibc.channel_genesis.ack_sequences += [{
  #         "port_id": $cw20_port,
  #         "channel_id": "channel-0",
  #         "sequence": "1"
  #       }, {
  #         "port_id": $ping_pong_port,
  #         "channel_id": "channel-1",
  #         "sequence": "1"
  #       }]' \
  #       $out/config/genesis.json | sponge $out/config/genesis.json

  #     jq \
  #      --arg ping_pong_port $PING_PONG_PORT \
  #      '.app_state.ibc.channel_genesis.channels += [{
  #         "state": 3,
  #         "ordering": 1,
  #         "counterparty": {
  #           "port_id": "ping-pong",
  #           "channel_id": "channel-1"
  #         },
  #         "connection_hops": ["connection-0"],
  #         "version": "ics20-1",
  #         "port_id": $ping_pong_port,
  #         "channel_id": "channel-1"
  #       }]' \
  #       $out/config/genesis.json | sponge $out/config/genesis.json

  #     jq \
  #       '.app_state.ibc.channel_genesis.next_channel_sequence = "2"' \
  #       $out/config/genesis.json | sponge $out/config/genesis.json

  #     jq \
  #       '.app_state.capability.index = "3"' \
  #       $out/config/genesis.json | sponge $out/config/genesis.json

  #     jq \
  #      --arg capability1 capabilities/ports/$CW20_PORT/channels/channel-0 \
  #      --arg capability2 capabilities/ports/$PING_PONG_PORT/channels/channel-1 \
  #       '.app_state.capability.owners += [{
  #         "index": "1",
  #         "index_owners": {
  #           "owners": [
  #             {
  #               "module": "ibc",
  #               "name": $capability1
  #             },
  #             {
  #               "module": "wasm",
  #               "name": $capability1
  #             }
  #           ]
  #         }
  #       },{
  #         "index": "2",
  #         "index_owners": {
  #           "owners": [
  #             {
  #               "module": "ibc",
  #               "name": $capability2
  #             },
  #             {
  #               "module": "wasm",
  #               "name": $capability2
  #             }
  #           ]
  #         }
  #       } ]' \
  #       $out/config/genesis.json | sponge $out/config/genesis.json

  #   '';

  alicePubkey = home:
    pkgs.runCommand "alice-pubkey" { buildInputs = [ pkgs.jq pkgs.moreutils ]; } ''
      export HOME=$(pwd)
      cp --no-preserve=mode -r ${home}/* .

      ALICE_ADDRESS=$(${nodeBin} keys list \
        --keyring-backend test \
        --home . \
        --output json \
        | jq '.[] | select(.name == "alice").address' --raw-output)

      ALICE_CANONICAL_KEY=$(${nodeBin} keys parse $ALICE_ADDRESS \
        --keyring-backend test \
        --home . \
        --output json | jq -r .bytes)

      echo -n "$ALICE_CANONICAL_KEY" > $out
    '';

  contractChecksum = contract:
    pkgs.runCommand "contract-checksum" { buildInputs = [ pkgs.moreutils ]; } ''
      for wasm in $(find ${contract} -name "*.wasm" -type f); do
        CHECKSUM=$(sha256sum $wasm | cut -f1 -d " ")
        echo "Found $wasm"
        echo "Checksum: $CHECKSUM"
        echo -n "$CHECKSUM" > $out
      done
    '';

  getContractAddress = creator: checksum: salt:
    pkgs.runCommand "get-contract-address" { buildInputs = [ pkgs.jq pkgs.devnet-utils ]; } ''
      export HOME=$(pwd)
      CANONICAL_ADDR=$(devnet-utils \
        compute \
        instantiate2-address \
        --creator "0x$(cat ${creator})" \
        --checksum "0x$(cat ${checksum})" \
        --salt "${salt}")
      ${nodeBin} \
        keys \
        parse "$CANONICAL_ADDR" \
        --output json | jq -r ".formats[0]" > $out
    '';

  addContractAddresses = { code, instances }: home:
    let
      checksum = contractChecksum code;
      creator = alicePubkey home;
    in
    pkgs.runCommand
      "${chainName}-add-contract-addresses"
      { buildInputs = [ pkgs.jq pkgs.moreutils ]; }
      ''
        export HOME=$(pwd)
        mkdir -p $out
        cp --no-preserve=mode -r ${home}/* $out
        for wasm in $(find ${code} -name "*.wasm" -type f); do
          mkdir -p $out/checksums
          echo -n $(cat "${checksum}") > "$out/checksums/$(basename $wasm .wasm)"
          mkdir -p $out/addresses
          ${builtins.concatStringsSep "\n" (pkgs.lib.imap0 (idx: { salt, ... }: ''
            echo -n "$(cat ${getContractAddress creator checksum salt})" > "$out/addresses/$(basename $wasm .wasm)_${builtins.toString idx}"
          '') instances)}
        done
      '';

  addLightClientCodeToGenesis = contract: home:
    pkgs.runCommand
      "${chainName}-add-light-client-contract-code-to-genesis"
      { buildInputs = [ pkgs.jq pkgs.moreutils ]; }
      ''
        export HOME=$(pwd)
        mkdir -p $out
        cp --no-preserve=mode -r ${home}/* $out

        # Generate the wasm client genesis state
        for wasm in $(find ${contract} -name "*.wasm" -type f); do
          echo "adding $wasm to genesis"

          base64 -w0 $wasm > encoded
          CHECKSUM=$(sha256sum $wasm | cut -f1 -d " ")
          # CODE_ID=$(echo -ne "codeId/$CHECKSUM" | base64 -w0)

          # echo "code id is '$CODE_ID'"

          mkdir -p $out/code-ids
          echo "$CHECKSUM" > "$out/code-ids/$(basename $wasm .wasm)"

           jq \
            --rawfile encoded_file encoded \
            '.app_state."08-wasm".contracts += [{
              "code_bytes": $encoded_file
            }]' \
            $out/config/genesis.json | sponge $out/config/genesis.json
        done
      '';

  addIbcContractCodesToGenesis = contracts: home:
    let
      addContract = { contract, idx }: home:
        pkgs.runCommand
          "${chainName}-add-ibc-contract-code-to-genesis"
          { buildInputs = [ pkgs.jq pkgs.moreutils pkgs.xxd ]; }
          ''
            export HOME=$(pwd)
            mkdir -p $out
            cp --no-preserve=mode -r ${home}/* $out

            ALICE_ADDRESS=$(${nodeBin} keys list \
              --keyring-backend test \
              --home $out \
              --output json \
              | jq '.[] | select(.name == "alice").address' --raw-output)

            for wasm in $(find ${contract} -name "*.wasm" -type f); do
              echo "adding $wasm to genesis"

              echo "code id is '${toString idx}'"

              mkdir -p $out/code-ids
              echo "${toString idx}" > "$out/code-ids/$(basename $wasm .wasm)"

              jq \
                --arg code_hash $(sha256sum $wasm | cut -f1 -d" " | xxd -r -p | base64) \
                --arg last_code_id_key $(echo -ne "\x04lastCodeId" | base64) \
                --arg creator $ALICE_ADDRESS \
                --rawfile encoded <(base64 -w0 $wasm) \
                '.app_state.wasm.codes += [{
                  "code_id": "${toString idx}",
                  "code_info": {
                    "code_hash": $code_hash,
                    "creator": $creator,
                    "instantiate_config": { "permission": "Everybody" }
                  },
                  "code_bytes": $encoded,
                  "pinned": true
                }]' \
                $out/config/genesis.json | sponge $out/config/genesis.json
            done
          '';

      home' = (pkgs.lib.foldl
        (h: contract: addContract contract h)
        home
        (pkgs.lib.imap1 (idx: c: { contract = c; inherit idx; }) contracts));
    in
    pkgs.runCommand
      "${chainName}-add-ibc-contract-codes-to-genesis"
      { buildInputs = [ pkgs.jq pkgs.moreutils ]; }
      ''
        export HOME=$(pwd)
        mkdir -p $out
        cp --no-preserve=mode -r ${home'}/* $out

          jq \
            --arg last_code_id_key $(echo -ne "\x04lastCodeId" | base64) \
            '.app_state.wasm.sequences += [{
              "id_key": $last_code_id_key,
              "value": "${toString ((builtins.length contracts) + 1)}",
            }]' \
            $out/config/genesis.json | sponge $out/config/genesis.json
      '';

  genesisHome = pkgs.lib.foldl
    (home: f: f home)
    (initHome null)
    (
      pkgs.lib.flatten [
        addAllKeysToKeyringAndGenesis

        applyGenesisOverwrites

        # add light clients
        (builtins.map addLightClientCodeToGenesis lightClients)

        # add ibc contracts
        enableAllClients

        (addIbcContractCodesToGenesis (builtins.map ({ code, ... }: code) cosmwasmContracts))

        (builtins.map addContractAddresses cosmwasmContracts)

        # add ibc connection
        # addIbcConnectionToGenesis
        # addIbcChannelToGenesis
      ]
    );

  devnet-home = pkgs.runCommand "${chainName}-home" { } ''
    mkdir $out
    cd $out

    export HOME=$(pwd)

    # Copy the read-only genesis we used to build the genesis file as the collect-gentxs command will overwrite it
    cp --no-preserve=mode -r ${genesisHome}/* .

    mkdir ./config/gentx
    ${builtins.concatStringsSep "\n" (builtins.genList (idx: ''
      cp ${mkValGentx idx} ./config/gentx/valgentx-${toString idx}.json
    '') validatorCount)}

    echo "collecting"
    # collect-gentxs was moved to a subcommand of genesis in sdk v50
    ${nodeBin} ${if sdkVersion >= 50 then "genesis" else ""} collect-gentxs --home . 2> /dev/null

    echo "validating"
    ${if sdkVersion < 50 then ''
      ${nodeBin} validate-genesis --home .
    '' else ''
      ${nodeBin} genesis validate --home .
    ''}
  '';

  mkValidatorHome = idx:
    pkgs.runCommand
      "${chainName}-validator_${toString idx}-home"
      { }
      ''
        mkdir $out
        cd $out


        cp --no-preserve=mode -RL ${devnet-home}/* $out
        cp --no-preserve=mode -L ${mkPrivValidatorKey idx} $out/config/priv_validator_key.json
        cp --no-preserve=mode -L ${mkNodeKey idx} $out/config/node_key.json

        cat $out/config/config.toml | grep "persistent_peers ="

        cat ${mkNodeId 0}

        # All nodes connect to node 0
        sed -i "s/persistent_peers = \".*\"/persistent_peers = \"$(cat ${mkNodeId 0})@${chainName}-0:26656\"/" $out/config/config.toml
        sed -i 's/chain-id = ""/chain-id = "${chainId}"/' $out/config/client.toml

        cat $out/config/config.toml | grep "persistent_peers ="
      '';

  mkNodeService = idx:
    {
      image = {
        enableRecommendedContents = true;
        contents = [
          pkgs.coreutils
          pkgs.curl
          node
          (mkValidatorHome idx)
        ] ++ extraPackages;
      };
      service = {
        tty = true;
        stop_signal = "SIGINT";
        ports = [
          # CometBLS JSONRPC 26657
          "${toString (26657 + portIncrease + idx)}:26657"
          # Cosmos SDK GRPC 9090
          "${toString (9090 + portIncrease + idx)}:9090"
          # Cosmos SDK REST 1317
          "${toString (1317 + portIncrease + idx)}:1317"
        ];
        command = [
          "sh"
          "-c"
          (''
            mkdir home

            cp --no-preserve=mode -RL ${mkValidatorHome idx}/* home

            mkdir ./tmp
            export TMPDIR=./tmp

          '' + (
            if startCommandOverwrite == null
            then
              ''
                ${nodeBin} comet show-node-id --home home

                ${nodeBin} \
                  start \
                  --home home \
                  $$params \
                  --rpc.laddr tcp://0.0.0.0:26657 \
                  --api.enable true \
                  --rpc.unsafe \
                  --api.address tcp://0.0.0.0:1317 \
                  --grpc.address 0.0.0.0:9090
              ''
            else
              startCommandOverwrite
          ))
        ];
        healthcheck = {
          interval = "5s";
          start_period = "20s";
          retries = 8;
          test = [
            "CMD-SHELL"
            ''
              curl http://127.0.0.1:26657/block?height=2 --fail || exit 1
            ''
          ];
        };
      };
    };
in
{
  inherit devnet-home;
  services = builtins.listToAttrs
    (builtins.genList
      (id: {
        name = "${chainName}-${toString id}";
        value = mkNodeService id;
      })
      validatorCount) // {
    "${chainName}-cosmwasm-deployer" = import ./services/cosmwasm-deployer.nix {
      inherit pkgs;
      inherit devnet-home;
      inherit node;
      inherit cosmwasmContracts;
      depends-on-node = "${chainName}-${toString 0}";
    };
  };
}
