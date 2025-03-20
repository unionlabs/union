import { Context } from "effect"
import type { Address } from "viem"

export class EvmChannelDestination extends Context.Tag("EvmChannelDestination")<
  EvmChannelDestination,
  { readonly ucs03address: Address; readonly channelId: number }
>() {}

export class EvmChannelSource extends Context.Tag("EvmChannelSource")<
  EvmChannelSource,
  { readonly ucs03address: Address; readonly channelId: number }
>() {}
