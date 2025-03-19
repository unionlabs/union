import { Context, Data } from "effect"
import type {
  PublicClient,
  ReadContractErrorType,
  WalletClient,
  WriteContractErrorType
} from "viem"

export class PublicSourceViemClient extends Context.Tag("PublicSourceViemClient")<
  PublicSourceViemClient,
  { readonly client: PublicClient }
>() {}

export class PublicDestinationViemClient extends Context.Tag("PublicDestinationViemClient")<
  PublicDestinationViemClient,
  { readonly client: PublicClient }
>() {}

/**
 * A neutral public client that can be used for general-purpose operations
 * that don't specifically target source or destination chains
 */
export class PublicViemClient extends Context.Tag("PublicViemClient")<
  PublicViemClient,
  { readonly client: PublicClient }
>() {}

/**
 * A wallet client that can be used for signing transactions
 */
export class ViemWalletClient extends Context.Tag("ViemWalletClient")<
  ViemWalletClient,
  { readonly client: WalletClient }
>() {}

export class ReadContractError extends Data.TaggedError("ReadContractError")<{
  cause: ReadContractErrorType
}> {}

export class WriteContractError extends Data.TaggedError("WriteContractError")<{
  cause: WriteContractErrorType
}> {}
