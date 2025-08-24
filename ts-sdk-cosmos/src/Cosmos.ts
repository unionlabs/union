/**
 * This module handles Cosmos related functionality.
 *
 * @since 2.0.0
 */
import { CosmWasmClient, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate"
import { FetchHttpClient } from "@effect/platform"
import { HttpClient, HttpClientRequest } from "@effect/platform"
import * as Ucs05 from "@unionlabs/sdk/Ucs05"
import * as Utils from "@unionlabs/sdk/Utils"
import { Context, Data, Effect, flow, Layer, pipe } from "effect"
import * as O from "effect/Option"

/**
 * @category models
 * @since 2.0.0
 */
export namespace Cosmos {
  /**
   * @category models
   * @since 2.0.0
   */
  export interface PublicClient {
    client: CosmWasmClient
  }

  /**
   * @category models
   * @since 2.0.0
   */
  export interface SigningClient {
    client: SigningCosmWasmClient
    address: string
  }

  /**
   * @category models
   * @since 2.0.0
   */
  export interface Channel {
    readonly ucs03address: string
    readonly channelId: number
  }
}

const clientLayer = <
  Id,
>(tag: Context.Tag<Id, Cosmos.PublicClient>) =>
(...options: Parameters<typeof CosmWasmClient.connect>): Layer.Layer<Id, ClientError> =>
  Layer.effect(
    tag,
    pipe(
      Effect.tryPromise({
        try: () => CosmWasmClient.connect(...options),
        catch: error => new ClientError({ cause: Utils.extractErrorDetails(error as Error) }),
      }),
      Effect.map((client) => ({ client })),
      Effect.timeout("10 seconds"),
      Effect.retry({ times: 5 }),
      Effect.mapError((e) => new ClientError({ cause: e as Error })),
    ),
  )

const signingClientLayer = <
  Id,
>(tag: Context.Tag<Id, Cosmos.SigningClient>) =>
(
  address: string,
  ...options: Parameters<typeof SigningCosmWasmClient.connectWithSigner>
): Layer.Layer<Id, ClientError> =>
  Layer.effect(
    tag,
    pipe(
      Effect.tryPromise({
        try: () => SigningCosmWasmClient.connectWithSigner(...options),
        catch: error => new ClientError({ cause: Utils.extractErrorDetails(error as Error) }),
      }),
      Effect.map((client) => ({ address, client })),
      Effect.timeout("10 seconds"),
      Effect.retry({ times: 5 }),
      Effect.mapError((e) => new ClientError({ cause: e as Error })),
    ),
  )

/**
 * @category context
 * @since 2.0.0
 */
export class ChannelDestination extends Context.Tag("@unionlabs/sdk/Cosmos/ChannelDestination")<
  ChannelDestination,
  Cosmos.Channel
>() {
  static Live = flow(
    ChannelDestination.of,
    Layer.succeed(this),
  )
}

/**
 * @category context
 * @since 2.0.0
 */
export class ChannelSource extends Context.Tag("@unionlabs/sdk/Cosmos/ChannelSource")<
  ChannelSource,
  Cosmos.Channel
>() {
  static Live = flow(
    ChannelSource.of,
    Layer.succeed(this),
  )
}

/**
 * Context for providing a CosmWasmClient for the source chain
 *
 * @category context
 * @since 2.0.0
 */
export class ClientSource extends Context.Tag("@unionlabs/sdk/Cosmos/ClientSource")<
  ClientSource,
  Cosmos.PublicClient
>() {
  static Live = clientLayer(this)
}

/**
 * Context for providing a CosmWasmClient for the destination chain
 *
 * @category context
 * @since 2.0.0
 */
export class ClientDestination extends Context.Tag("@unionlabs/sdk/Cosmos/ClientDestination")<
  ClientDestination,
  Cosmos.PublicClient
>() {
  static Live = clientLayer(this)
}

/**
 * A neutral CosmWasmClient that can be used for general-purpose operations
 * that don't specifically target source or destination chains
 *
 * @category context
 * @since 2.0.0
 */
export class Client extends Context.Tag("@unionlabs/sdk/Cosmos/Client")<
  Client,
  Cosmos.PublicClient
>() {
  static Live = clientLayer(this)
}

/**
 * Context for providing a SigningCosmWasmClient
 *
 * @category context
 * @since 2.0.0
 */
export class SigningClient extends Context.Tag("@unionlabs/sdk/Cosmos/SigningClient")<
  SigningClient,
  Cosmos.SigningClient
>() {
  static Live = signingClientLayer(this)
  static FromSigningClient = (address: string, client: SigningCosmWasmClient) =>
    Layer.effect(
      this,
      Effect.succeed({
        client,
        address,
      }),
    )
}

/**
 * @category context
 * @since 2.0.0
 */
export class DestinationConfig extends Context.Tag("@unionlabs/sdk/Cosmos/DestinationConfig")<
  DestinationConfig,
  Cosmos.Channel
>() {}

/**
 * Error type for CosmWasm client failures
 *
 * @category errors
 * @since 2.0.0
 */
export class ClientError extends Data.TaggedError("@unionlabs/sdk/Cosmos/ClientError")<{
  cause: unknown
}> {}

/**
 * Error type for CosmWasm contract query failures
 *
 * @category errors
 * @since 2.0.0
 */
export class QueryContractError
  extends Data.TaggedError("@unionlabs/sdk/Cosmos/QueryContractError")<{
    address: string
    cause: unknown
  }>
{}

/**
 * Error type for CosmWasm contract execution failures
 *
 * @category errors
 * @since 2.0.0
 */
export class ExecuteContractError
  extends Data.TaggedError("@unionlabs/sdk/Cosmos/ExecuteContractError")<{
    message: string
    cause: unknown
  }>
{}

/**
 * Error when fetching the latest block height
 *
 * @category errors
 * @since 2.0.0
 */
export class GetHeightError extends Data.TaggedError("@unionlabs/sdk/Cosmos/GetHeightError")<{
  cause: unknown
}> {}

/**
 * Error when fetching a balance at latest height
 *
 * @category errors
 * @since 2.0.0
 */
export class GetBalanceError extends Data.TaggedError("@unionlabs/sdk/Cosmos/GetBalanceError")<{
  cause: unknown
}> {}

/**
 * A type-safe wrapper around CosmWasm's queryContract that handles error cases
 * and returns an Effect with proper type inference.
 *
 * @param client - The CosmWasmClient to use for the contract query
 * @param contractAddress - The address of the contract to query
 * @param queryMsg - The query message to send to the contract
 * @returns An Effect that resolves to the properly typed return value
 *
 * @category utils
 * @since 2.0.0
 */
export const queryContract = <T = unknown>(
  contractAddress: string,
  queryMsg: Record<string, unknown>,
) =>
  pipe(
    Client,
    Effect.andThen(({ client }) =>
      Effect.tryPromise({
        try: async () => {
          const result = await client.queryContractSmart(contractAddress, queryMsg)
          return result as T
        },
        catch: error =>
          new QueryContractError({
            address: contractAddress,
            cause: Utils.extractErrorDetails(error as Error),
          }),
      })
    ),
    Effect.timeout("10 seconds"),
    Effect.retry({ times: 5 }),
    Effect.annotateLogs({
      contractAddress,
      queryMsg,
    }),
  )

/**
 * A type-safe wrapper around CosmWasm's executeContract that handles error cases
 * and returns an Effect with proper type inference.
 *
 * @param senderAddress - The address of the sender executing the contract
 * @param contractAddress - The address of the contract to execute
 * @param msg - The execute message to send to the contract
 * @param funds - Optional funds to send with the transaction
 * @returns An Effect that resolves to the execution result
 *
 * @category utils
 * @since 2.0.0
 */
export const executeContract = (
  senderAddress: string,
  contractAddress: string,
  msg: Record<string, unknown>,
  funds?: ReadonlyArray<{ denom: string; amount: string }>,
) =>
  Effect.andThen(SigningClient, ({ client }) =>
    Effect.tryPromise({
      try: () =>
        client.execute(
          senderAddress,
          contractAddress,
          msg,
          "auto",
          undefined,
          funds,
        ),
      catch: error =>
        new ExecuteContractError({
          cause: Utils.extractErrorDetails(error as Error),
          message: (error as Error).message,
        }),
    }))

/**
 * Wrap CosmWasmClient.getHeight() in an Effect
 * @see https://cosmos.github.io/cosmjs/latest/cosmwasm-stargate/classes/CosmWasmClient.html#getHeight
 *
 * @category utils
 * @since 2.0.0
 */
export function getChainHeight(
  client: CosmWasmClient,
) {
  return Effect.tryPromise({
    try: () => client.getHeight(),
    catch: (err) => new GetHeightError({ cause: Utils.extractErrorDetails(err as Error) }),
  }).pipe(
    Effect.timeout("10 seconds"),
    Effect.retry({ times: 5 }),
  )
}

/**
 * Wrap CosmWasmClient.getBalance() in an Effect
 *
 * @category utils
 * @since 2.0.0
 */
export const getBalanceNow = Effect.fn("getBalanceNow")((
  address: `${string}1${string}`,
  denom: string,
) =>
  pipe(
    Client,
    Effect.andThen(({ client }) =>
      Effect.tryPromise({
        try: () => client.getBalance(address, denom),
        catch: (err) => new GetBalanceError({ cause: Utils.extractErrorDetails(err as Error) }),
      })
    ),
  )
)

// .pipe(
//     Effect.timeout("10 seconds"),
//     Effect.retry({ times: 5 }),
//   )

/**
 * Error type for HttpRequest execution failures
 *
 * @category errors
 * @since 2.0.0
 */
export class HttpRequestFailed extends Data.TaggedError("@unionlabs/sdk/Cosmos/HttpRequestFailed")<{
  status: number
  body: unknown
}> {}

/**
 * Query a contract at an explicit block height.
 *
 * @category utils
 * @since 2.0.0
 */
export function queryContractSmartAtHeight<T = unknown>(
  restEndpoint: string,
  contractAddress: string,
  queryMsg: Record<string, unknown>,
  height: number,
) {
  const base = restEndpoint.replace(/\/+$/, "")
  const encoded = btoa(JSON.stringify(queryMsg))
  const url = `${base}/cosmwasm/wasm/v1/contract/${contractAddress}/smart/${encoded}`
  return pipe(
    Effect.gen(function*() {
      const request = HttpClientRequest.get(url).pipe(
        HttpClientRequest.setHeaders({
          "Content-Type": "application/json",
          "x-cosmos-block-height": height.toString(),
        }),
      )

      const client = yield* HttpClient.HttpClient
      const response = yield* client.execute(request)
      const data = yield* response.json

      return data as T
    }),
    Effect.timeout("10 seconds"),
    Effect.retry({ times: 5 }),
    Effect.catchAll((err) =>
      err instanceof HttpRequestFailed
        ? Effect.fail(err)
        : Effect.fail(
          new QueryContractError({
            address: contractAddress,
            cause: err,
          }),
        )
    ),
  )
}

/**
 * Fetch an account's balance for a denom at a specific block height.
 *
 * @category utils
 * @since 2.0.0
 */
export function getBalanceAtHeight(
  restEndpoint: string,
  address: string,
  denom: string,
  height: number,
) {
  const base = restEndpoint.replace(/\/+$/, "")
  const url = `${base}/cosmos/bank/v1beta1/balances/${address}`
  return pipe(
    Effect.gen(function*() {
      const request = HttpClientRequest.get(url).pipe(
        HttpClientRequest.setHeaders({
          "Content-Type": "application/json",
          "x-cosmos-block-height": height.toString(),
        }),
      )

      const client = yield* HttpClient.HttpClient
      const response = yield* client.execute(request)
      const raw = yield* response.json

      const data = raw as {
        balances: Array<{ denom: string; amount: string }>
      }

      const entry = data.balances.find((b) => b.denom === denom)
      return entry ? BigInt(entry.amount) : null
    }),
    Effect.timeout("10 seconds"),
    Effect.retry({ times: 5 }),
    Effect.catchAll((err) =>
      err instanceof HttpRequestFailed
        ? Effect.fail(err)
        : Effect.fail(
          new QueryContractError({
            address,
            cause: err,
          }),
        )
    ),
  )
}

/**
 * Interface for CW20 token metadata
 *
 * @category models
 * @since 2.0.0
 */
export interface Cw20TokenInfo {
  name: string
  symbol: string
  decimals: number
  total_supply: string
}

/**
 * Interface for CW20 token balance response
 *
 * @category models
 * @since 2.0.0
 */
export interface Cw20BalanceResponse {
  balance: string
}

/**
 * Interface for CW20 token balance response
 *
 * @category models
 * @since 2.0.0
 */
export interface Cw20AllowanceResponse {
  allowance: number
  expiration: any // XXX
}

/**
 * Read CW20 token metadata (name, symbol, decimals, total_supply)
 * @param contractAddress The address of the CW20 token contract
 * @returns An Effect that resolves to the token metadata
 *
 * @category utils
 * @since 2.0.0
 */
export const readCw20TokenInfo = (contractAddress: string) =>
  queryContract<Cw20TokenInfo>(contractAddress, { token_info: {} })

/**
 * Read CW20 token total_supply
 * @param contractAddress The address of the CW20 token contract
 * @returns An Effect that resolves to the token total supply
 *
 * @category utils
 * @since 2.0.0
 */
export const readCw20TotalSupply = (contractAddress: string) =>
  pipe(
    queryContract<Cw20TokenInfo>(contractAddress, {
      token_info: {},
    }),
    Effect.map(x => x.total_supply),
  )

/**
 * Read the balance of a CW20 token for a specific address
 * @param rest The rest url
 * @param contractAddress The address of the CW20 token contract
 * @param address The address to check the balance for
 * @param height Height of the chain
 * @returns An Effect that resolves to the token balance
 *
 * @category utils
 * @since 2.0.0
 */
export const readCw20BalanceAtHeight = (
  rest: string,
  contractAddress: string,
  address: string,
  height: number,
) =>
  Effect.gen(function*() {
    const resp = yield* queryContractSmartAtHeight<{ data: { balance: string } }>(
      rest,
      contractAddress,
      {
        balance: {
          address,
        },
      },
      height,
    ).pipe(
      Effect.provide(FetchHttpClient.layer),
      Effect.tapErrorCause((cause) => Effect.logError("cosmos.readCw20BalanceAtHeight", cause)),
    )
    return resp.data.balance
  })

/**
 * Read CW20 token total_supply
 * @param rest The rest url
 * @param contractAddress The address of the CW20 token contract
 * @param height Height of the chain
 * @returns An Effect that resolves to the token total supply
 *
 * @category utils
 * @since 2.0.0
 */
export const readCw20TotalSupplyAtHeight = (
  rest: string,
  contractAddress: string,
  height: number,
) =>
  Effect.gen(function*() {
    const resp = yield* queryContractSmartAtHeight<
      { data: { name: string; symbol: string; decimals: number; total_supply: string } }
    >(rest, contractAddress, {
      token_info: {},
    }, height).pipe(
      Effect.provide(FetchHttpClient.layer),
      Effect.tapErrorCause((cause) => Effect.logError("cosmos.readCw20TotalSupplyAtHeight", cause)),
    )
    return resp.data.total_supply
  })

/**
 * Read the balance of a CW20 token for a specific address
 * @param contractAddress The address of the CW20 token contract
 * @param address The address to check the balance for
 * @returns An Effect that resolves to the token balance
 *
 * @category utils
 * @since 2.0.0
 */
export const readCw20Balance = (contractAddress: string, address: string) =>
  pipe(
    queryContract<Cw20BalanceResponse>(contractAddress, {
      balance: {
        address,
      },
    }),
    Effect.map(x => x.balance),
  )

/**
 * Read the allowance of a CW20 token for a specific addresses
 * @param contract The address of the CW20 token contract
 * @param owner The owner of the token
 * @param spender The spender who will spend the token
 * @returns An Effect that resolves to the token allowance
 *
 * @category utils
 * @since 2.0.0
 */
export const readCw20Allowance = (
  contract: Ucs05.CosmosDisplay,
  owner: Ucs05.CosmosDisplay,
  spender: Ucs05.CosmosDisplay,
) =>
  pipe(
    queryContract<Cw20AllowanceResponse>(contract.address, {
      allowance: {
        owner: owner.address,
        spender: spender.address,
      },
    }),
    Effect.map(x => x.allowance),
  )

/**
 * Increase the allowance of a CW20 token for a specific spender.
 *
 * @param contractAddress The address of the CW20 token contract.
 * @param senderAddress The address of the token owner (the one increasing the allowance).
 * @param spenderAddress The address of the spender who is allowed to spend the tokens.
 * @param amount The amount by which to increase the allowance (as a string).
 * @param expires Optional expiration for the increased allowance.
 * @returns An Effect that resolves to the execution result.
 *
 * @category utils
 * @since 2.0.0
 */
export const writeCw20IncreaseAllowance = (
  contractAddress: string,
  senderAddress: string,
  spenderAddress: string,
  amount: string,
) =>
  executeContract(senderAddress, contractAddress, {
    increase_allowance: {
      spender: spenderAddress,
      amount,
    },
  })

/**
 * Checks whether a denom is a native token or CW20.
 * @param denom The denom address to check.
 * @returns An Effect that resolves to true if native, false if CW20.
 *
 * @category utils
 * @since 2.0.0
 */
export const isDenomNative = (denom: string) =>
  Effect.gen(function*() {
    const client = (yield* Client).client

    return yield* readCw20TokenInfo(denom).pipe(
      Effect.provideService(Client, { client }),
      Effect.map(() => false),
      Effect.catchAllCause(() => Effect.succeed(true)),
    )
  })

/**
 * @category utils
 * @since 2.0.0
 */
export const channelBalance = (path: bigint, token: string) =>
  pipe(
    ChannelDestination,
    Effect.andThen((config) =>
      queryContract(config.ucs03address, {
        get_channel_balance: {
          channel_id: config.channelId,
          path: path,
          denom: token,
        },
      })
    ),
  )

/**
 * @category utils
 * @since 2.0.0
 */
export const channelBalanceAtHeight = (rest: string, path: bigint, token: string, height: number) =>
  Effect.gen(function*() {
    const config = yield* ChannelDestination
    const resp = yield* queryContractSmartAtHeight<{ data: string | undefined }>(
      rest,
      config.ucs03address,
      {
        get_channel_balance: {
          channel_id: config.channelId,
          path,
          denom: token,
        },
      },
      height,
    ).pipe(
      Effect.tap((resp) => Effect.log({ resp })),
      Effect.provide(FetchHttpClient.layer),
      Effect.tapErrorCause((cause) => Effect.logError("cosmos.channelBalanceAtHeight", cause)),
    )
    return yield* O.fromNullable(resp.data)
  })
