import { Context } from "effect"

export class CosmosChannelDestination extends Context.Tag("CosmosChannelDestination")<
  CosmosChannelDestination,
  { readonly ucs03address: string; readonly channelId: number }
>() {}

export class CosmosChannelSource extends Context.Tag("CosmosChannelSource")<
  CosmosChannelSource,
  { readonly ucs03address: string; readonly channelId: number }
>() {}
