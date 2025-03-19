import { Context, Data } from "effect"
import type {
  PublicClient,
  ReadContractErrorType,
  WalletClient,
  WriteContractErrorType
} from "viem"

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
  { readonly client: WalletClient }
>() {}

export class ReadContractError extends Data.TaggedError("ReadContractError")<{
  cause: ReadContractErrorType
}> {}

export class WriteContractError extends Data.TaggedError("WriteContractError")<{
  cause: WriteContractErrorType
}> {}
