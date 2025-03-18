import { Context } from "effect"
import type { PublicClient } from "viem"

export class PublicSourceViemClient extends Context.Tag("PublicSourceViemClient")<
  PublicSourceViemClient,
  { readonly client: PublicClient }
>() {}

export class PublicDestinationViemClient extends Context.Tag("PublicDestinationViemClient")<
  PublicDestinationViemClient,
  { readonly client: PublicClient }
>() {}
