import { Schedule, Data, Effect, Option, Schema } from "effect"
import type { DurationInput } from "effect/Duration"
import type { TimeoutException } from "effect/Cause"
import { RawTokenBalance, TokenRawAmount, type TokenRawDenom } from "$lib/schema/token"
import type { Chain } from "$lib/schema/chain"
// You can import a retry schedule specific for Aptos; here we assume one exists.
import { aptosBalanceRetrySchedule } from "$lib/constants/schedules"
import { getPublicClient } from "$lib/services/aptos/clients"
import { Aptos, AptosConfig, Network, MoveVector } from "@aptos-labs/ts-sdk"

export type FetchAptosBalanceError =
  | FetchAptosNativeBalanceError
  | FetchAptosTokenBalanceError

export class FetchAptosNativeBalanceError extends Data.TaggedError("FetchAptosNativeBalanceError")<{
  cause: unknown
}> {}

export class FetchAptosTokenBalanceError extends Data.TaggedError("FetchAptosTokenBalanceError")<{
  cause: unknown
}> {}


/**
 * createAptosBalanceQuery
 *
 * This function creates an effect-based pipeline that repeatedly fetches the balance for a given Aptos token.
 * It takes the same parameters as the EVM version.
 *
 * - If `tokenAddress` is `"native"`, it fetches the native (gas) balance from an Aptos fullnode REST API.
 * - Otherwise, it uses a GraphQL query (similar to your old updateBalancesAptos logic) to fetch token balances,
 *   then filters for tokens with metadata.token_standard === "v2", and extracts the balance for the given token.
 *
 * The fetched balance is then passed to the provided `writeData` callback (wrapped in your RawTokenBalance schema)
 * and any errors are reported via `writeError`.
 */
export const createAptosBalanceQuery = ({
  chain,
  tokenAddress,
  walletAddress,
  refetchInterval,
  writeData,
  writeError
}: {
  chain: Chain
  tokenAddress: TokenRawDenom
  walletAddress: string
  refetchInterval: DurationInput
  writeData: (data: RawTokenBalance) => void
  writeError: (error: Option.Option<FetchAptosBalanceError>) => void
}) => {
  const fetcherPipeline = Effect.gen(function* (_) {
    yield* Effect.log(
      `starting aptos balance fetcher for ${chain.universal_chain_id}:${walletAddress}:${tokenAddress}`
    )

    let balance: bigint
    const aptosClient = yield* getPublicClient(chain)

        // For tokens (v2), query the GraphQL endpoint.
    const query = `
      query CoinsData($owner_address: String, $limit: Int, $offset: Int) {
        current_fungible_asset_balances(
          where: {owner_address: {_eq: $owner_address}}
          limit: $limit
          offset: $offset
        ) {
          amount
          asset_type
          metadata {
            name
            decimals
            symbol
            token_standard
          }
        }
      }
    `
    const variables = {
      owner_address: walletAddress,
      limit: 200,
      offset: 0
    }

    const response = yield* Effect.retry(
      Effect.tryPromise({
        try: () =>
          fetch("https://indexer.testnet.movementnetwork.xyz/v1/graphql", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ query, variables })
          }).then(res => res.json()),
        catch: err => new FetchAptosTokenBalanceError({ cause: err })
      }),
      aptosBalanceRetrySchedule
    )

    console.info("response: ", response)

    // Extract tokens from the response and filter for "v2" tokens.
    const tokens = response.data.current_fungible_asset_balances as Array<any>
    const aptosTokens = tokens.filter(
      token => token.metadata.token_standard === "v2" && token.asset_type === tokenAddress 
    )
    // Find the token that matches the provided tokenAddress.
    const tokenData = aptosTokens.find(token => token.asset_type === tokenAddress)
    balance = tokenData ? BigInt(tokenData.amount) : 0n  

    writeData(RawTokenBalance.make(Option.some(TokenRawAmount.make(balance))))
    writeError(Option.none())
  }).pipe(
    Effect.tapError(error => Effect.sync(() => writeError(Option.some(error)))),
    Effect.catchAll(() => Effect.succeed(null))
  )

  return Effect.repeat(
    fetcherPipeline,
    Schedule.addDelay(Schedule.repeatForever, () => refetchInterval)
  )
}
