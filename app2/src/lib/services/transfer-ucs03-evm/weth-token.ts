import { Effect } from "effect";
import { type Hex } from "viem";
import { ucs03ZkgmAbi } from "$lib/abi/ucs03.ts";
import type { Channel } from "$lib/schema/channel.ts";
import { getQuoteToken } from "./quote-token";
import type {Chain} from "$lib/schema/chain.ts";
import {getPublicClient} from "$lib/services/evm/clients.ts"; // Adjust path

export const getWethQuoteToken = (
  sourceChain: Chain,
  ucs03Address: Hex,
  channel: Channel
) => Effect.gen(function* () {
  const publicClient = yield* getPublicClient(sourceChain)

  const wethAddress = yield* Effect.tryPromise({
    try: () => publicClient.readContract({
      address: ucs03Address,
      abi: ucs03ZkgmAbi,
      functionName: "weth" as const,
      args: []
    }) as Promise<Hex>,
    catch: (error) => new Error("Failed to get WETH address from zkgm contract", { cause: error })
  });

  const quoteResult = yield* getQuoteToken(sourceChain, wethAddress, channel, "evm").pipe(
    Effect.orElse(() => Effect.succeed({ type: "NO_WETH_QUOTE" as const }))
  );

  if (quoteResult.type === "NO_WETH_QUOTE") {
    return { type: "NO_WETH_QUOTE" as const };
  }

  return { wethQuoteToken: quoteResult.quote_token };
});