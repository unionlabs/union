{ ... }: {
  perSystem = { crane, lib, dbg, pkgs, ... }:
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
    in
    {
      packages = minimal.packages // mainnet.packages // {
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
  
              TARGET_SLOT=$(echo "$line" | jq .client_message.data.consensus_update.attested_header.beacon.slot -r)

              echo "processing line: $I, slot: $TARGET_SLOT"

              filename="$TARGET_SLOT"

              next_sync_committee=$(echo "$line" | jq .client_message.data.consensus_update.next_sync_committee_branch)
              if [ "$next_sync_committee" != "null" ]; then
                filename="sync_committee_update-$filename.json"
              else
                filename="finality_update-$filename.json"
              fi

              echo "$line" | jq > "$OUTPUT_PATH/$filename"

              I=$((I+1))

            done 
          '';
        };
      };
      checks = minimal.checks // mainnet.checks;
    };
}
