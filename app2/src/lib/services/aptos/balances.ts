import { Data, Effect, Option } from "effect"
import { RawTokenBalance, TokenRawAmount, type TokenRawDenom } from "$lib/schema/token"
import type { Chain } from "$lib/schema/chain"
// You can import a retry schedule specific for Aptos; here we assume one exists.
import { aptosBalanceRetrySchedule } from "$lib/constants/schedules"
import { getPublicClient } from "$lib/services/aptos/clients"
import type { Aptos, AptosApiError } from "@aptos-labs/ts-sdk"
import { extractErrorDetails } from "@unionlabs/sdk/utils"

export type FetchAptosBalanceError = FetchAptosTokenBalanceError

export class FetchAptosTokenBalanceError extends Data.TaggedError("FetchAptosTokenBalanceError")<{
  cause: unknown
}> {}

const fetchFABalance = ({
  aptosClient,
  tokenAddress,
  walletAddress
}: {
  aptosClient: Aptos
  tokenAddress: TokenRawDenom
  walletAddress: string
}) =>
  Effect.tryPromise({
    try: () =>
      aptosClient.view({
        payload: {
          function: `0x1::primary_fungible_store::balance`,
          typeArguments: ["0x1::fungible_asset::Metadata"],
          functionArguments: [walletAddress.toString(), tokenAddress.toString()]
        }
      }),
    catch: err =>
      new FetchAptosTokenBalanceError({ cause: extractErrorDetails(err as AptosApiError) })
  })

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
export const fetchAptosBalance = ({
  chain,
  tokenAddress,
  walletAddress
}: {
  chain: Chain
  tokenAddress: TokenRawDenom
  walletAddress: string
}) => {
  return Effect.gen(function* (_) {
    const aptosClient = yield* getPublicClient(chain)

    yield* Effect.log(
      `starting aptos balance fetcher for ${chain.universal_chain_id}:${walletAddress}:${tokenAddress}`
    )

    let balance: bigint

    const fetchBalance = fetchFABalance({ aptosClient, tokenAddress, walletAddress })
    const balance_request = yield* Effect.retry(fetchBalance, aptosBalanceRetrySchedule)

    balance = BigInt(balance_request[0])

    return RawTokenBalance.make(Option.some(TokenRawAmount.make(balance)))
  })
}
