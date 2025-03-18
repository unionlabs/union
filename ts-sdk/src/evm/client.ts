import { Context, Data } from "effect"
import type { PublicClient, ReadContractErrorType } from "viem"

export class PublicSourceViemClient extends Context.Tag("PublicSourceViemClient")<
  PublicSourceViemClient,
  { readonly client: PublicClient }
>() {}

export class PublicDestinationViemClient extends Context.Tag("PublicDestinationViemClient")<
  PublicDestinationViemClient,
  { readonly client: PublicClient }
>() {}

export class ReadContractError extends Data.TaggedError("ReadContractError")<{
  cause: ReadContractErrorType
}> {}
