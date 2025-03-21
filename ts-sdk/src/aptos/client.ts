import { Context, Data, Effect } from "effect"
import { extractErrorDetails } from "../utils/extract-error-details.js"
import { Aptos, type AptosConfig, type AptosApiError } from "@aptos-labs/ts-sdk"

export class AptosPublicClientSource extends Context.Tag("AptosPublicClientSource")<
  AptosPublicClientSource,
  { readonly client: Aptos }
>() {}

export class AptosPublicClientDestination extends Context.Tag("AptosPublicClientDestination")<
  AptosPublicClientDestination,
  { readonly client: Aptos }
>() {}

/**
 * A neutral public client that can be used for general-purpose operations
 * that don't specifically target source or destination chains
 */
export class AptosPublicClient extends Context.Tag("AptosPublicClient")<
  AptosPublicClient,
  { readonly client: Aptos }
>() {}

// /**
//  * A wallet client that can be used for signing transactions
//  */
// export class AptosWalletClient extends Context.Tag("AptosWalletClient")<
// AptosWalletClient,
//   {
//     readonly client: WalletClient
//     readonly account: Account
//     readonly chain: Chain
//   }
// >() {}

export class AptosError extends Data.TaggedError("AptosError")<{
  cause: AptosApiError
}> {}

/** Thrown if creating a public Aptos client fails. */
export class CreatePublicAptosClientError extends Data.TaggedError("CreatePublicAptosClientError")<{
  cause: unknown
}> {}

/** Thrown if creating a wallet-based Aptos client fails. */
export class CreateWalletAptosClientError extends Data.TaggedError("CreateWalletAptosClientError")<{
  cause: unknown
}> {}

export const createAptosPublicClient = (
  parameters: AptosConfig
): Effect.Effect<Aptos, CreatePublicAptosClientError> =>
  Effect.try({
    try: () => new Aptos(parameters),
    catch: err =>
      new CreatePublicAptosClientError({
        cause: extractErrorDetails(err as CreatePublicAptosClientError)
      })
  })

export const createAptosWalletClient = (
  parameters: AptosConfig
): Effect.Effect<Aptos, CreateWalletAptosClientError> =>
  Effect.try({
    try: () => new Aptos(parameters),
    catch: err =>
      new CreateWalletAptosClientError({
        cause: extractErrorDetails(err as CreateWalletAptosClientError)
      })
  })
