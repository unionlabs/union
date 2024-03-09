import { getBalance } from "#/query.ts";
import { unionActions } from "#/actions";
import { isValidChainId } from "#/constants";
import {
  createPublicClient,
  fallback,
  formatUnits,
  http,
  walletActions,
} from "viem";

/**
 * bun ./scripts/balance.ts \
 *   --chainId '11155111' | '6' \
 *   --address '0x3a7c1964ea700Ee19887c747C72e68F84Cb9C5DD'
 *
 * e.g.
 * bun ./scripts/balance.ts --chainId 11155111 --address 0x3a7c1964ea700Ee19887c747C72e68F84Cb9C5DD
 * bun ./scripts/balance.ts --chainId 6 --address union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv
 */

main().catch((_) => {
  console.error(_);
  process.exit(1);
});

async function main() {
  const [chainFlag, chainId, addressFlag, address] = process.argv.slice(2);

  if (!(chainFlag && chainId && addressFlag && address))
    throw new Error(
      "Usage: bun ./scripts/balance.ts --chain <chain> --address <address>"
    );

  if (!isValidChainId(chainId)) throw new Error(`Invalid chain: ${chainId}`);
  const client = createPublicClient({
    transport: fallback([
      http(process.env.SEPOLIA_RPC_URL),
      http("https://ethereum-sepolia.publicnode.com"),
    ]),
  })
    .extend(walletActions)
    .extend(unionActions);

  // @ts-expect-error
  const balance = await getBalance(client, { chainId, address: address });

  console.log({
    [address]: {
      chainId,
      balance: {
        bigint: balance,
        number:
          chainId === "11155111"
            ? formatUnits(balance, 18)
            : formatUnits(balance, 6),
      },
    },
  });
}
