import { Context } from "effect"
import type { Hex } from "viem"

export class SuiFungibleAssetOrderDetails extends Context.Tag("SuiFungibleAssetOrderDetails")<
  SuiFungibleAssetOrderDetails,
  {
    readonly typename_t: Hex
    readonly ibc_store: Hex
    readonly relay_store: Hex
    readonly coin: Hex
    readonly metadata: Hex
  }
>() {}
