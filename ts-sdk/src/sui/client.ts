import { SuiClient, SuiClientOptions } from "@mysten/sui/client"
import { Context, Data, Effect } from "effect"
import {
  type Account,
  type Chain,
  type CreatePublicClientErrorType,
  createWalletClient,
  type CreateWalletClientErrorType,
  type ReadContractErrorType,
  type WalletClient,
  type WalletClientConfig,
  type WriteContractErrorType,
} from "viem"
import { extractErrorDetails } from "../utils/extract-error-details.js"
import { Ed25519Keypair } from '@mysten/sui/keypairs/ed25519';

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
  { readonly client: SuiClient }
>() {}

/**
 * A wallet client that can be used for signing transactions
 */
export class SuiWalletClient extends Context.Tag("SuiWalletClient")<
  SuiWalletClient,
  {
    readonly client: SuiClient
    readonly signer: Ed25519Keypair
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

export class CreateSuiWalletClientError extends Data.TaggedError("CreateSuiWalletClientError")<{
  cause: CreateWalletClientErrorType
}> {}

export const createSuiPublicClient = (
  parameters: SuiClientOptions,
): Effect.Effect<SuiPublicClient, CreateSuiPublicClientError> =>
  Effect.try({
    try: () => new SuiClient(parameters),
    catch: err =>
      new CreateSuiPublicClientError({
        cause: extractErrorDetails(err as CreatePublicClientErrorType),
      }),
  })

// the constructor function
export const createSuiWalletClient = (
  options: SuiClientOptions,
  signer: Ed25519Keypair
): Effect.Effect<SuiWalletClient, CreateSuiWalletClientError> =>
  Effect.try({
    try: () => ({
      client: new SuiClient(options),
      signer,
    }),
    catch: (err) =>
      new CreateSuiWalletClientError({
        cause: extractErrorDetails(err as CreateWalletClientErrorType),
      }),
  });