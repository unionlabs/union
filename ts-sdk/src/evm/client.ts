import { Context, Data, Effect } from "effect"
import {
  createPublicClient,
  createWalletClient,
  type Account,
  type Chain,
  type CreatePublicClientErrorType,
  type CreateWalletClientErrorType,
  type PublicClient,
  type PublicClientConfig,
  type ReadContractErrorType,
  type WalletClient,
  type WalletClientConfig,
  type WriteContractErrorType
} from "viem"
import { extractErrorDetails } from "../utils/extract-error-details.js"

export class ViemPublicClientSource extends Context.Tag("ViemPublicClientSource")<
  ViemPublicClientSource,
  { readonly client: PublicClient }
>() {}

export class ViemPublicClientDestination extends Context.Tag("ViemPublicClientDestination")<
  ViemPublicClientDestination,
  { readonly client: PublicClient }
>() {}

/**
 * A neutral public client that can be used for general-purpose operations
 * that don't specifically target source or destination chains
 */
export class ViemPublicClient extends Context.Tag("ViemPublicClient")<
  ViemPublicClient,
  { readonly client: PublicClient }
>() {}

/**
 * A wallet client that can be used for signing transactions
 */
export class ViemWalletClient extends Context.Tag("ViemWalletClient")<
  ViemWalletClient,
  {
    readonly client: WalletClient
    readonly account: Account
    readonly chain: Chain
  }
>() {}

export class ReadContractError extends Data.TaggedError("ReadContractError")<{
  cause: ReadContractErrorType
}> {}

export class WriteContractError extends Data.TaggedError("WriteContractError")<{
  cause: WriteContractErrorType
}> {}

export class CreateViemPublicClientError extends Data.TaggedError("CreateViemPublicClientError")<{
  cause: CreatePublicClientErrorType
}> {}

export class CreateViemWalletClientError extends Data.TaggedError("CreateViemWalletClientError")<{
  cause: CreateWalletClientErrorType
}> {}

export const createViemPublicClient = (
  parameters: PublicClientConfig
): Effect.Effect<PublicClient, CreateViemPublicClientError> =>
  Effect.try({
    try: () => createPublicClient(parameters),
    catch: err =>
      new CreateViemPublicClientError({
        cause: extractErrorDetails(err as CreatePublicClientErrorType)
      })
  })

export const createViemWalletClient = (
  parameters: WalletClientConfig
): Effect.Effect<WalletClient, CreateViemWalletClientError> =>
  Effect.try({
    try: () => createWalletClient(parameters),
    catch: err =>
      new CreateViemWalletClientError({
        cause: extractErrorDetails(err as CreateWalletClientErrorType)
      })
  })
