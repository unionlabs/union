{ ... }: {
  perSystem = { crane, lib, dbg, pkgs, writeShellApplicationWithArgs, ... }:
    let
      mkEthLc = chain-spec: crane.buildWasmContract {
        crateDirFromRoot = "light-clients/ethereum-light-client";
        features = [ chain-spec ];
        additionalTestSrcFilter = path: _:
          (lib.hasPrefix "light-clients/ethereum-light-client/src/test" path)
          && (lib.strings.hasSuffix ".json" path);
      };

      minimal = mkEthLc "minimal";
      mainnet = mkEthLc "mainnet";

      parse-test-data = pkgs.writeShellApplication {
        name = "parse-test-data";
        runtimeInputs = [ pkgs.jq ];
        text = ''
          I=0
          while read -r line; do 
            # TODO(aeryz): add this in case we need a limit
            # if [[ $A -eq 10 ]]; then 
            #   break; 
            # fi;
            I=$((I+1))
                
            TARGET_SLOT=$(echo "$line" | jq .client_message.data.consensus_update.attested_header.beacon.slot -r)

            echo "processing line: $I, slot: $TARGET_SLOT"

            filename="$TARGET_SLOT"

            next_sync_committee=$(echo "$line" | jq .client_message.data.consensus_update.next_sync_committee_branch)
            if [ "$next_sync_committee" != "null" ]; then
              filename="sync_committee_update-$filename.json"
            else
              filename="finality_update-$filename.json"
            fi

            echo "$line" | jq .client_message.data > "$UPDATES_PATH/$filename"
          done 
        '';
      };

      # NOTE(aeryz): This script currently act as a convenient way to call `eth_getProof`. When we add query
      # client state capability to `ucli`, we will extend this to also query a given client and construct the
      # full test data. 
      fetch-membership-data = writeShellApplicationWithArgs {
        name = "fetch-membership-data";
        runtimeInputs = [ pkgs.jq ];
        arguments = [{
          arg = "execution_endpoint";
          required = true;
        }
          {
            arg = "contract";
            required = true;
            help = "The address of the contract that we want to read the storage of";
          }
          {
            arg = "commitment_key";
            required = true;
            help = "The slot where the value is stored at";
          }
          {
            arg = "at";
            default = "latest";
            help = "The height of the block that we fetch the data from";
          }];
        text = ''
          curl -s --header "Content-Type: application/json" \
               -X POST \
               --data '
                 {
                   "jsonrpc":"2.0", 
                   "method": "eth_getProof", 
                   "params": [
                     "'"$argc_contract"'", 
                     ["'"$argc_commitment_key"'"], 
                     "'"$argc_at"'"
                   ], 
                   "id": 1
                 }' "$argc_execution_endpoint"  | jq
        '';
      };
    in
    {
      packages = minimal.packages // mainnet.packages // {
        inherit parse-test-data;
        inherit fetch-membership-data;
      };
      checks = minimal.checks // mainnet.checks;
    };
}
