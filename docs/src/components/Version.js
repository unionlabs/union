import versions from "../../../versions.json";

export default function Version(chainId, genesis) {
  if (chainId == "union-testnet-4")
    if (genesis) return versions["union-testnet-4"][0];
    else return versions["union-testnet-4"].at(-1);
  else throw new Error(`Unknown chainId: ${chainId}`);
}
