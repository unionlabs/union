import { SuiClient, SuiClientOptions } from "@mysten/sui/client"
import { Ed25519Keypair } from "@mysten/sui/keypairs/ed25519"
import { Context, Data, Effect } from "effect"
import { extractErrorDetails } from "../utils/extract-error-details.js"

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

export class SuiReadContractError extends Data.TaggedError("SuiReadContractError")<{
  cause: unknown
}> {}

export class SuiWriteContractError extends Data.TaggedError("SuiWriteContractError")<{
  cause: unknown
}> {}

export class SuiCreateWalletClientErrorType extends Data.TaggedError("SuiCreateWalletClientErrorType")<{
  cause: unknown
}> {}

export class SuiCreatePublicClientErrorType extends Data.TaggedError("SuiCreatePublicClientErrorType")<{
  cause: unknown
}> {}

export class CreateSuiPublicClientError extends Data.TaggedError("CreateSuiPublicClientError")<{
  cause: SuiCreatePublicClientErrorType
}> {}

export class CreateSuiWalletClientError extends Data.TaggedError("CreateSuiWalletClientError")<{
  cause: SuiCreateWalletClientErrorType
}> {}

export const createSuiPublicClient = (
  parameters: SuiClientOptions,
) =>
  Effect.try({
    try: () => new SuiClient(parameters),
    catch: err =>
      new CreateSuiPublicClientError({
        cause: extractErrorDetails(err as SuiCreatePublicClientErrorType),
      }),
  })

export const createSuiWalletClient = (
  options: SuiClientOptions,
  signer: Ed25519Keypair,
) =>
  Effect.try({
    try: () => ({
      client: new SuiClient(options),
      signer,
    }),
    catch: (err) =>
      new CreateSuiWalletClientError({
        cause: extractErrorDetails(err as SuiCreateWalletClientErrorType),
      }),
  })
