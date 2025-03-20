import { Context, Data, Effect } from "effect"
import {
  CosmWasmClient,
  SigningCosmWasmClient,
  type SigningCosmWasmClientOptions
} from "@cosmjs/cosmwasm-stargate"
import { extractErrorDetails } from "../utils/extract-error-details.js"

/**
 * Context for providing a CosmWasmClient for the source chain
 */
export class CosmWasmClientSource extends Context.Tag("CosmWasmClientSource")<
  CosmWasmClientSource,
  { client: CosmWasmClient }
>() {}

/**
 * Context for providing a CosmWasmClient for the destination chain
 */
export class CosmWasmClientDestination extends Context.Tag("CosmWasmClientDestination")<
  CosmWasmClientDestination,
  { client: CosmWasmClient }
>() {}

/**
 * A neutral CosmWasmClient that can be used for general-purpose operations
 * that don't specifically target source or destination chains
 */
export class CosmWasmClientContext extends Context.Tag("CosmWasmClientContext")<
  CosmWasmClientContext,
  { client: CosmWasmClient }
>() {}

/**
 * Context for providing a SigningCosmWasmClient
 */
export class SigningCosmWasmClientContext extends Context.Tag("SigningCosmWasmClientContext")<
  SigningCosmWasmClientContext,
  {
    client: SigningCosmWasmClient
    address: string
  }
>() {}

/**
 * Error type for CosmWasm client failures
 */
export class CosmWasmClientError extends Data.TaggedError("CosmWasmClientError")<{
  cause: unknown
}> {}

/**
 * Creates a CosmWasmClient from a given RPC endpoint
 *
 * @param rpcEndpoint - The RPC endpoint to connect to
 * @returns An Effect that resolves to a CosmWasmClient
 */
export const createCosmWasmClient = (rpcEndpoint: string) =>
  Effect.tryPromise({
    try: () => CosmWasmClient.connect(rpcEndpoint),
    catch: error => new CosmWasmClientError({ cause: extractErrorDetails(error as Error) })
  }).pipe(Effect.timeout("10 seconds"), Effect.retry({ times: 5 }))

/**
 * Creates a SigningCosmWasmClient from a given RPC endpoint and wallet
 *
 * @param rpcEndpoint - The RPC endpoint to connect to
 * @param signer - The signer to use for transactions
 * @returns An Effect that resolves to a SigningCosmWasmClient
 */
export const createSigningCosmWasmClient = (
  rpcEndpoint: string,
  signer: any,
  options: SigningCosmWasmClientOptions
) =>
  Effect.tryPromise({
    try: () => SigningCosmWasmClient.connectWithSigner(rpcEndpoint, signer, options),
    catch: error => new CosmWasmClientError({ cause: extractErrorDetails(error as Error) })
  }).pipe(Effect.timeout("10 seconds"), Effect.retry({ times: 5 }))
