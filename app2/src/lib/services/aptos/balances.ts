import { Schedule, Data, Effect, Option, Schema } from "effect"
import type { DurationInput } from "effect/Duration"
import type { TimeoutException } from "effect/Cause"
import { RawTokenBalance, TokenRawAmount, type TokenRawDenom } from "$lib/schema/token"
import type { Chain } from "$lib/schema/chain"
// You can import a retry schedule specific for Aptos; here we assume one exists.
import { aptosBalanceRetrySchedule } from "$lib/constants/schedules"
import { getPublicClient } from "$lib/services/aptos/clients"
import { Aptos, AptosConfig, Network, MoveVector } from "@aptos-labs/ts-sdk"
import type { GetBalanceErrorType, ReadContractErrorType } from "viem"

export type FetchAptosBalanceError =
  | FetchAptosTokenBalanceError

export class FetchAptosTokenBalanceError extends Data.TaggedError("FetchAptosTokenBalanceError")<{
  cause: unknown
}> {}

// const fetchFABalance = ({
//   aptosClient,
//   tokenAddress,
//   walletAddress
// }: {
//   aptosClient: Aptos
//   tokenAddress: TokenRawDenom
//   walletAddress: string
// }) =>

//   Effect.tryPromise({
//     try: () =>
//       aptosClient.view({
//         payload: {
//           function: `${channel.destination_port_id}::ibc_app::predict_wrapped_token`,
//           typeArguments: [],
//           functionArguments: [
//             0, // path
//             channel.destination_channel_id,
//             MoveVector.U8(base_token)
//           ]
//         }
//       }),
//     catch: err => new ReadContractError({ cause: err as ReadContractErrorType })
//   })

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
            functionArguments: [
              walletAddress.toString(),
              tokenAddress.toString()
            ]
          }
        }),
      catch: err => new FetchAptosTokenBalanceError({ cause: err as ReadContractErrorType })
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
    console.info("aptosClient: ", aptosClient)


    const fetchBalance = fetchFABalance({aptosClient, tokenAddress, walletAddress})
    const balance_request = yield* Effect.retry(fetchBalance, aptosBalanceRetrySchedule)

    balance = BigInt(balance_request[0])
   
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
