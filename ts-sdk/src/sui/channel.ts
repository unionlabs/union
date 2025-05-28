import { Context } from "effect"
import type { Address } from "viem"

export class SuiChannelDestination extends Context.Tag("SuiChannelDestination")<
  SuiChannelDestination,
  { readonly ucs03address: Address; readonly channelId: number }
>() {}

export class SuiChannelSource extends Context.Tag("SuiChannelSource")<
  SuiChannelSource,
  { readonly ucs03address: Address; readonly channelId: number }
>() {}
