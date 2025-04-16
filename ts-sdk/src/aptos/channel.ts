import { Context } from "effect"
import type { Address } from "viem"

export class AptosChannelDestination extends Context.Tag("AptosChannelDestination")<
  AptosChannelDestination,
  { readonly ucs03address: Address; readonly channelId: number }
>() {}

export class AptosChannelSource extends Context.Tag("AptosChannelSource")<
  AptosChannelSource,
  { readonly ucs03address: Address; readonly channelId: number }
>() {}
