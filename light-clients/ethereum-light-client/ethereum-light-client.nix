_: {
  perSystem =
    {
      crane,
      lib,
      dbg,
      pkgs,
      ensure-wasm-client-type,
      mkCi,
      ...
    }:
    let
      mkEthLc =
        chain-spec:
        crane.buildWasmContract {
          crateDirFromRoot = "light-clients/ethereum-light-client";
          features = [ (pkgs.lib.strings.toLower chain-spec) ];
          checks = [
            (file_path: ''
              ${ensure-wasm-client-type {
                inherit file_path;
                type = "Ethereum${chain-spec}";
              }}
            '')
          ];
        };

      minimal = mkEthLc "Minimal";
      mainnet = mkEthLc "Mainnet";

      gen-eth-lc-update-test-data = mkCi false (
        pkgs.writeShellApplicationWithArgs {
          name = "parse-test-data";
          runtimeInputs = [ pkgs.jq ];
          arguments = [
            {
              arg = "output_path";
              required = true;
              help = "The output directory to put the update data";
            }
            {
              arg = "finality_update_per_period";
              default = "99999999";
              help = "The maximum limit of finality update data to generate per sync committee period";
            }
            {
              arg = "test_data";
              required = true;
              help = "The exported test data that is going to be processed";
            }
          ];
          text = ''
            I=0
            FINALITY=0
            NEXT=0
            n_finality=0
            last_processed_slot=0
            while read -r line; do 
              I=$((I+1))
                  
              line=$(echo "$line" | jq .Sequence[0].Lc.EthereumMinimal.data.Msg.UpdateClient.msg)
              TARGET_SLOT=$(echo "$line" | jq .client_message.data.consensus_update.attested_header.beacon.slot -r)
              if (( last_processed_slot == TARGET_SLOT )); then
                continue
              fi
              last_processed_slot="$TARGET_SLOT"
              echo "processing line: $I, slot: $TARGET_SLOT"

              next_sync_committee=$(echo "$line" | jq .client_message.data.consensus_update.next_sync_committee_branch)
              if [ "$next_sync_committee" != "null" ]; then
                echo "[ i ] Generated next sync committee update."
                NEXT=$((NEXT+1))
                n_finality=0
              else
                if (( n_finality >= argc_finality_update_per_period )); then
                  continue
                fi

                n_finality=$((n_finality+1))
              fi

              echo "$line" | jq .client_message.data > "$argc_output_path/$TARGET_SLOT.json"
            done < "$argc_test_data"

            echo "[ + ] Generated $FINALITY finality updates and $NEXT sync committee updates."
          '';
        }
      );
    in
    {
      packages =
        minimal.packages
        // mainnet.packages
        // {
          inherit gen-eth-lc-update-test-data;
        };
      checks = minimal.checks // mainnet.checks;
    };
}
