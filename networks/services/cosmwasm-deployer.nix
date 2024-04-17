{ pkgs, devnet-home, node, depends-on-node, instantiations, ... }:
let
  cosmwasm-deployer =
    pkgs.writeShellApplication {
      name = "cosmwasm-deployer";
      runtimeInputs = [ node ];
      text = ''
        mkdir -p /tmp
        ${builtins.concatStringsSep "\n" (pkgs.lib.imap0 (idx: {code-id, salt, label, message }:
          ''
            ${pkgs.lib.getExe node} \
             tx \
             wasm \
             instantiate2 \
             --no-admin \
             ${builtins.toString code-id} \
             '${builtins.toJSON message}' \
             "${salt}" \
             --label "${label}" \
             --home ${devnet-home} \
             --gas=auto \
             --gas-adjustment=1.4 \
             --keyring-backend test \
             --from alice \
             -y \
             --node http://${depends-on-node}:26657 \
             --generate-only > ./msg${builtins.toString idx}.json
          ''
        ) instantiations)}

        # Merge and sign messages
        ${pkgs.lib.getExe node} \
          tx \
          sign-batch ${builtins.concatStringsSep " " (builtins.genList (id: "./msg${builtins.toString id}.json") (builtins.length instantiations))} \
          --append \
          --home ${devnet-home} \
          --gas=auto \
          --gas-adjustment=1.4 \
          --keyring-backend test \
          --from alice \
          --node http://${depends-on-node}:26657 \
          -y > tx.json

        # Broadcast signed transaction
        ${pkgs.lib.getExe node} \
          tx \
          broadcast tx.json \
          --home ${devnet-home} \
          --gas=auto \
          --gas-adjustment=1.4 \
          --keyring-backend test \
          --from alice \
          --node http://${depends-on-node}:26657 \
          -y
      '';
    };
in
{
  image = {
    enableRecommendedContents = true;
    contents = [
      pkgs.coreutils
      pkgs.curl
      cosmwasm-deployer
    ];
  };
  service = {
    tty = true;
    stop_signal = "SIGINT";
    command = [ (pkgs.lib.getExe cosmwasm-deployer) ];
    depends_on = {
      "${depends-on-node}" = {
        condition = "service_healthy";
      };
    };
  };
}
