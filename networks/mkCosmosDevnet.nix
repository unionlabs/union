{
  pkgs,
  dbg,
  ...
}:
{
  node,
  chainId,
  chainName,
  denom,
  keyType,
  validatorCount,
  portIncrease,
  genesisOverwrites ? { },
  lightClients ? [ ],
  cosmwasmContracts ? [ ],
  startCommandOverwrite ? null,
  extraPackages ? [ ],
  sdkVersion ? 50,
  sdkPatchVersion ? 0,
  has08Wasm ? false,
}:
assert (builtins.isString chainId);
assert (builtins.isString chainName);
assert (builtins.isString denom);
assert (builtins.isString keyType);
assert (builtins.isInt portIncrease);
assert (builtins.isInt validatorCount);
assert (
  pkgs.lib.assertOneOf "sdkVersion" sdkVersion [
    47
    50
    52
  ]
);
assert (builtins.isBool has08Wasm);
let
  devKeyMnemonics = {
    alice = "wine parrot nominee girl exchange element pudding grow area twenty next junior come render shadow evidence sentence start rough debate feed all limb real";
    bob = "gun more barrel helmet velvet people alter depth bargain use isolate pear before frown already limb sweet response legal invest stand barrel stone conduct";
    charlie = "young soup enroll tornado mercy athlete tray resist limit spare address license cargo quantum panda useful clog autumn shoot observe input next across movie";
    dave = "allow where void replace vocal cabbage can expose rival danger stomach noodle faculty cart surround cash rice kite audit slight ten bicycle dance middle";
    erin = "hard educate knock ketchup salon obey debate one other impose smoke spoon pull describe cactus talk other merit joy great critic canvas scene lounge";
    frank = "over floor explain will stereo camera subway park pilot trick good exchange foot violin shop kite educate bracket shoulder fancy denial ill era battle";
    gabe = "expose case pulp tone typical orbit hybrid total chest vanish exhibit boat eager glue soldier december author crucial abstract fruit clock sun away organ";
    hussein = "tribe scale pulp napkin cactus female velvet fever outdoor mimic summer obey tornado giant tennis cube game cover midnight general thrive boring situate west";
    ibiza = "fork slender display army write mixture deputy giant siren injury tornado culture spice message mandate anxiety blade pigeon actress ivory sorry latin panther high";
    jake = "where devote brisk game include voice evil name fancy pizza nice find rely convince honey gas follow milk certain tennis style material reform electric";
  };

  nodeBin = pkgs.lib.getExe node;

  genScriptForEachVal = f: ''
    ${builtins.concatStringsSep "\n" (builtins.genList f validatorCount)}
  '';

  mkNodeMnemonic =
    idx:
    assert (builtins.isInt idx);
    pkgs.runCommand "${chainName}-mnemonic_${toString idx}" { buildInputs = [ pkgs.devnet-utils ]; } ''
      echo "keygen start"
      devnet-utils keygen mnemonic $(echo ${toString idx} | sha256sum - | cut -d' ' -f1) > $out
      echo "Keygen done"

      echo "validator ${toString idx} mnemonic: $(cat $out)"
    '';

  mkNodeKey =
    idx:
    assert (builtins.isInt idx);
    pkgs.runCommand "${chainName}-node-key_${toString idx}" { buildInputs = [ pkgs.devnet-utils ]; } ''
      NODE_KEY=$(devnet-utils keygen key --key-type ed25519 "$(cat ${mkNodeMnemonic idx})" | tr -d '\n')

      echo "validator ${toString idx} node_key: $NODE_KEY"

      echo "{\"priv_key\":{\"type\":\"tendermint/PrivKeyEd25519\",\"value\":\"$NODE_KEY\"}}" > $out
    '';

  mkNodeId =
    idx:
    assert (builtins.isInt idx);
    pkgs.runCommand "${chainId}-node-id_${toString idx}" { buildInputs = [ ]; } ''
      export HOME=$(pwd)

      cp -r --no-preserve=mode ${initHome idx}/* .

      cp ${mkNodeKey idx} ./config/node_key.json

      node_id="$(${nodeBin} tendermint show-node-id --home . 2>&1)"
      echo $node_id | tr -d '\n' > $out
    '';

  mkPrivValidatorKey =
    idx:
    assert (builtins.isInt idx);
    pkgs.runCommand "${chainName}-priv-validator-key_${toString idx}" { buildInputs = [ ]; } ''
      export HOME=$(pwd)

      cp -r --no-preserve=mode ${initHome idx}/* .

      mv ./config/priv_validator_key.json $out
      echo "created valkey-${toString idx}: $(cat $out)"
    '';

  mkValGentx =
    idx:
    assert (builtins.isInt idx);
    pkgs.runCommand "${chainName}-valgentx_${toString idx}"
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
          ${
            if (sdkVersion >= 50 || (sdkVersion >= 47 && sdkPatchVersion >= 8)) then "genesis" else ""
          } gentx \
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

  initHome =
    idx:
    pkgs.runCommand "${chainName}-genesis-home" { buildInputs = [ ]; } ''
      export HOME=$(pwd)
      mkdir -p $out

      echo "sdk version: ${toString sdkVersion}"
      cat ${mkNodeMnemonic (if idx == null then 0 else idx)} | ${nodeBin} \
        init \
        testnet ${
          # idk man
          pkgs.lib.optionalString (
            chainName != "osmosis" && (sdkVersion > 50 || sdkVersion < 50)
          ) ''--default-denom ${denom}''
        } \
        ${pkgs.lib.optionalString (sdkVersion >= 52) ''--consensus-key-algo ${keyType}''} \
        --chain-id ${chainId} \
        --home $out \
        --recover

      sed -i 's# "stake"# "${denom}"#g' $out/config/genesis.json
    '';

  addDevKeyToKeyringAndGenesis =
    name: mnemonic: home:
    pkgs.runCommand "${chainName}-add-dev-key-${name}"
      {
        buildInputs = [
          pkgs.jq
          pkgs.moreutils
        ];
      }
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
          ${
            if (sdkVersion >= 50 || (sdkVersion >= 47 && sdkPatchVersion >= 8)) then "genesis" else ""
          } add-genesis-account \
          ${name} \
          10000000000000000000000000${denom} \
          --keyring-backend test \
          --home $out
      '';

  addValoperKeyToKeyringAndGenesis =
    idx: home:
    assert (builtins.isInt idx);
    pkgs.runCommand "${chainName}-valkey_${toString idx}" { buildInputs = [ ]; } ''
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
        ${
          if (sdkVersion >= 50 || (sdkVersion >= 47 && sdkPatchVersion >= 8)) then "genesis" else ""
        } add-genesis-account \
        valoper-${toString idx} \
        10000000000000000000000000${denom} \
        --keyring-backend test \
        --home $out
    '';

  addAllKeysToKeyringAndGenesis =
    home:
    pkgs.lib.foldl (home: f: f home) home (
      pkgs.lib.flatten [
        (pkgs.lib.mapAttrsToList addDevKeyToKeyringAndGenesis devKeyMnemonics)
        (builtins.genList addValoperKeyToKeyringAndGenesis validatorCount)
      ]
    );

  applyGenesisOverwrites =
    home:
    let
      overwrites = builtins.toFile "overwrite.json" (builtins.toJSON genesisOverwrites);
    in
    pkgs.runCommand "${chainName}-apply-genesis-overwrites" { buildInputs = [ pkgs.jq ]; } ''
      mkdir -p $out
      cp --no-preserve=mode -r ${home}/* $out
      jq -s '.[0] * .[1]' ${home}/config/genesis.json ${overwrites} > merge.json
      mv merge.json $out/config/genesis.json
    '';

  setValidatorPubkey =
    home:
    pkgs.runCommand "${chainName}-set-validator-pubkey"
      {
        buildInputs = [
          pkgs.jq
          pkgs.moreutils
        ];
      }
      ''
        export HOME=$(pwd)
        mkdir -p $out
        cp --no-preserve=mode -r ${home}/* $out

        jq \
         '.consensus.params.validator.pub_key_types = ["${keyType}"]' \
          $out/config/genesis.json | sponge $out/config/genesis.json
      '';

  enablePBTS =
    home:
    pkgs.runCommand "${chainName}-set-validator-pubkey"
      {
        buildInputs = [
          pkgs.jq
          pkgs.moreutils
        ];
      }
      ''
        export HOME=$(pwd)
        mkdir -p $out
        cp --no-preserve=mode -r ${home}/* $out

        jq \
         '.consensus.params.feature.vote_extensions_enable_height = "0"' \
          $out/config/genesis.json | sponge $out/config/genesis.json

        jq \
         '.consensus.params.feature.pbts_enable_height = "1"' \
          $out/config/genesis.json | sponge $out/config/genesis.json
      '';

  genesisHome = pkgs.lib.foldl (home: f: f home) (initHome null) (
    pkgs.lib.flatten [
      addAllKeysToKeyringAndGenesis

      applyGenesisOverwrites

      setValidatorPubkey

      enablePBTS
    ]
  );

  devnet-home = pkgs.runCommand "${chainName}-home" { } ''
    mkdir $out
    cd $out

    export HOME=$(pwd)

    # Copy the read-only genesis we used to build the genesis file as the collect-gentxs command will overwrite it
    cp --no-preserve=mode -r ${genesisHome}/* .

    mkdir ./config/gentx

    ${genScriptForEachVal (idx: ''
      cp ${mkValGentx idx} ./config/gentx/valgentx-${toString idx}.json
    '')}

    echo "collecting"
    # collect-gentxs was moved to a subcommand of genesis in sdk v50
    ${nodeBin} ${
      if (sdkVersion >= 50 || (sdkVersion >= 47 && sdkPatchVersion >= 8)) then "genesis" else ""
    } collect-gentxs --home . 2> /dev/null

    echo "validating"
    ${
      if (sdkVersion >= 50) then
        ''
          ${nodeBin} genesis validate --home .
        ''
      else if (sdkVersion >= 47 && sdkPatchVersion >= 8) then
        ''
          ${nodeBin} genesis validate-genesis --home .
        ''
      else
        ''
          # ${nodeBin} validate-genesis --home .
        ''
    }

    sed -i 's/chain-id = ""/chain-id = "${chainId}"/' $out/config/client.toml

    sed -i 's/max_body_bytes = 1000000/max_body_bytes = 100000000/' $out/config/config.toml
    sed -i 's/max_tx_bytes = 1048576/max_tx_bytes = 10485760/' $out/config/config.toml
    sed -i 's/cors_allowed_origins = \[\]/cors_allowed_origins = \["*"\]/' $out/config/config.toml
  '';

  mkValidatorHome =
    idx:
    pkgs.runCommand "${chainName}-validator_${toString idx}-home" { } ''
      mkdir $out
      cd $out


      cp --no-preserve=mode -RL ${devnet-home}/* $out
      cp --no-preserve=mode -L ${mkPrivValidatorKey idx} $out/config/priv_validator_key.json
      cp --no-preserve=mode -L ${mkNodeKey idx} $out/config/node_key.json

      cat ${mkNodeId 0}

      # all nodes connect to node 0
      sed -i "s/persistent_peers = \".*\"/persistent_peers = \"$(cat ${mkNodeId 0})@${chainName}-0:26656\"/" $out/config/config.toml
    '';

  mkNodeService = idx: {
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
        # Cosmos SDK pprof
        "${toString (6060 + portIncrease + idx)}:6060"
      ];
      command = [
        "sh"
        "-c"
        (
          ''
            mkdir home

            cp --no-preserve=mode -RL ${mkValidatorHome idx}/* home

            mkdir ./tmp
            export TMPDIR=./tmp

          ''
          + (
            # --cpu-profile CPU_PROFILE \
            if startCommandOverwrite == null then
              ''
                ${nodeBin} comet show-node-id --home home

                # --wasm.skip_wasmvm_version_check \

                ${nodeBin} \
                  start \
                  --home home \
                  $$params \
                  --api.enabled-unsafe-cors \
                  --rpc.pprof_laddr        0.0.0.0:6060 \
                  --rpc.laddr              tcp://0.0.0.0:26657 \
                  --rpc.unsafe \
                  --api.enable             true \
                  --api.address            tcp://0.0.0.0:1317 \
                  --api.rpc-max-body-bytes 100000000 \
                  --grpc.address           0.0.0.0:9090 \
                  --minimum-gas-prices     "0${denom}" \
                  --log_level rpc-server:warn,x/wasm:debug,*:info
              ''
            else
              startCommandOverwrite
          )
        )
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

  upload-wasm-light-client =
    let
      rpc_endpoint = "http://localhost:${toString (26657 + portIncrease)}";
      # re-bind this so we can get proper syntax highlighting in the text arg
      writeShellApplication = pkgs.writeShellApplicationWithArgs;
    in
    writeShellApplication {
      name = "devnet-${chainName}-upload-wasm-light-client";
      runtimeInputs = [ node ];
      arguments = [
        {
          arg = "wasm_blob";
          type = "arg";
          help = "Path to the wasm blob to upload";
          required = true;
        }
      ];
      text = ''
        # this value isn't exposed anywhere, so read the abci store directly
        prop_id=$(("0x$(curl --silent "${rpc_endpoint}"'/abci_query?path="store/gov/key"&data=0x03' | jq '.result.response.value' -r | base64 --decode | hexdump -v -e '/1 "%02x"')"))

        echo "prop_id: $prop_id"

        ${nodeBin} tx ibc-wasm store-code "$argc_wasm_blob" --title "$argc_wasm_blob" --summary "$argc_wasm_blob" --deposit 100000${denom} --from valoper-0 --home ${devnet-home} --keyring-backend test --gas 100000000${denom} --gas-adjustment 3 -y --node "${rpc_endpoint}"

        until ${nodeBin} query gov proposal "$prop_id" --node "${rpc_endpoint}"; do echo "prop $prop_id not up yet"; sleep 1; done

        sleep 7

        ${nodeBin} tx gov deposit "$prop_id" 1000000000${denom} --from valoper-0 --home ${devnet-home} --keyring-backend test --gas auto --gas-adjustment 2 -y --node "${rpc_endpoint}"

        sleep 7

        ${genScriptForEachVal (idx: ''
          ${nodeBin} tx gov vote "$prop_id" yes --from valoper-${toString idx} --home ${devnet-home} --keyring-backend test -y --gas auto --gas-adjustment 2 --node "${rpc_endpoint}"
        '')}

        echo "contract uploaded, checksum: $(sha256sum "$argc_wasm_blob" | cut -d " " -f 1)"
      '';
    };
in
{
  inherit devnet-home;
  scripts =
    if has08Wasm then
      {
        "devnet-${chainName}-upload-wasm-light-client" = upload-wasm-light-client;
      }
    else
      { };
  services =
    builtins.listToAttrs (
      builtins.genList (id: {
        name = "${chainName}-${toString id}";
        value = mkNodeService id;
      }) validatorCount
    )
    // {
      "${chainName}-cosmwasm-deployer" = import ./services/cosmwasm-deployer.nix {
        inherit pkgs;
        inherit devnet-home;
        inherit node;
        inherit cosmwasmContracts;
        depends-on-node = "${chainName}-${toString 0}";
      };
    };
}
