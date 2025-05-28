import { Context, Data, Effect } from "effect"
import {
  type Account,
  type Chain,
  createPublicClient,
  type CreatePublicClientErrorType,
  createWalletClient,
  type CreateWalletClientErrorType,
  type PublicClient,
  type PublicClientConfig,
  type ReadContractErrorType,
  type WalletClient,
  type WalletClientConfig,
  type WriteContractErrorType,
} from "viem"
import { extractErrorDetails } from "../utils/extract-error-details.js"
import { SuiClient, SuiClientOptions } from '@mysten/sui/client';

export class SuiPublicClientSource extends Context.Tag("SuiPublicClientSource")<
SuiPublicClientSource,
  { readonly client: SuiClient }
>() {}

export class SuiPublicClientDestination extends Context.Tag("SuiPublicClientDestination")<
  SuiPublicClientDestination,
  { readonly client: SuiClient }
>() {}

/**
 * A neutral public client that can be used for general-purpose operations
 * that don't specifically target source or destination chains
 */
export class SuiPublicClient extends Context.Tag("SuiPublicClient")<
  SuiPublicClient,
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

export class CreateSuiPublicClientError extends Data.TaggedError("CreateSuiPublicClientError")<{
  cause: CreatePublicClientErrorType
}> {}

export class CreateViemWalletClientError extends Data.TaggedError("CreateViemWalletClientError")<{
  cause: CreateWalletClientErrorType
}> {}

export const createSuiPublicClient = (
  parameters: SuiClientOptions,
): Effect.Effect<PublicClient, CreateSuiPublicClientError> =>
  Effect.try({
    try: () => new SuiClient(parameters),
    catch: err =>
      new CreateSuiPublicClientError({
        cause: extractErrorDetails(err as CreatePublicClientErrorType),
      }),
  })

export const createViemWalletClient = (
  parameters: WalletClientConfig,
): Effect.Effect<WalletClient, CreateViemWalletClientError> =>
  Effect.try({
    try: () => createWalletClient(parameters),
    catch: err =>
      new CreateViemWalletClientError({
        cause: extractErrorDetails(err as CreateWalletClientErrorType),
      }),
  })
