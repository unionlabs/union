import { Effect } from "effect"
import type { Address, Hex, ReadContractErrorType } from "viem"
import { PublicDestinationViemClient, ReadContractError } from "./client.js"
import { ucs03abi } from "./abi/ucs03.js"

export const quoteToken = (baseToken: Hex, ucs03address: Address, destinationChannelId: number) =>
  Effect.gen(function* () {
    let client = (yield* PublicDestinationViemClient).client

    const predictedQuoteToken = yield* Effect.tryPromise({
      try: () =>
        client.readContract({
          address: ucs03address,
          abi: ucs03abi,
          functionName: "predictWrappedToken",
          args: [0n, destinationChannelId, baseToken]
        }),
      catch: error => new ReadContractError({ cause: error as ReadContractErrorType })
    }).pipe(Effect.map(([address]) => address))

    return predictedQuoteToken
  })
