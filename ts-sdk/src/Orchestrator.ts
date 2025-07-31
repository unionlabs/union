/**
req1 = SourceChain -> DestinationChain -> Channel[]

head Channel[] -> Channel

type FullQuoteToken = (QuoteToken, metadataType, metadata)

req2 = SourceChain -> Channel -> BaseToken -> FullQuoteToken[]

head FullQuoteToken[] -> FullQuoteToken

Chanel -> BaseToken -> FullQuoteToken -> FAOv2



# assuming canonical channel and canonical quote token


SourceChain -> DestinationChain -> BaseToken -> FAOv2


Requirements = SourceWalletClient | GraphqlAPI



# Three cases for req2

req2 = BaseToken -> Channel -> FullQuoteToken[]


## Case 1: Unwrapping

If BaseToken is actually a wrapped representation from Channel, req2 will return the metadataType = unwrap

## Case 2: Completely new asset

OnchainMetadata { Name, Symbol, Decimals }
TBD

## Case 3: preexisting wrapped asset

we submit metadatatype = image and we provide for metadata the image.
*/

import { Context, Effect } from "effect"
import * as Brand from "effect/Brand"
import * as S from "effect/Schema"
import { Chain } from "./schema/chain.js"
import { Channel } from "./schema/channel.js"

const BaseToken = S.String.pipe(
  S.brand("QuoteToken"),
)
type BaseToken = typeof BaseToken.Type

const QuoteToken = S.String.pipe(
  S.brand("QuoteToken"),
)
type QuoteToken = typeof QuoteToken.Type

const MetadataType = S.String.pipe(
  S.brand("MetadataType"),
)
type MetadataType = typeof MetadataType.Type

const Metadata = S.Struct({}).pipe(
  S.brand("Metadata"),
)
type Metadata = typeof Metadata.Type

const FAOv2 = S.Tuple(S.Any).pipe(
  S.brand("FungibleAssetOrderV2"),
)
type FAOv2 = typeof FAOv2.Type

class WalletClient extends Context.Tag("@unionlabs/sdk/Wallet")<
  WalletClient,
  {
    readonly sign: (order: FAOv2) => Effect.Effect<void, never, never>
  }
>() {}

class GraphQL extends Context.Tag("@unionlabs/sdk/GraphQL")<
  GraphQL,
  {
    readonly getChains: Effect.Effect<readonly Chain[], never, never>

    readonly getChannels: (
      source: SourceChain,
      destination: DestinationChain,
    ) => Effect.Effect<readonly Channel[], never, never>

    readonly getQuoteTokens: (
      source: SourceChain,
      channel: Channel,
      base: BaseToken,
    ) => Effect.Effect<readonly FullQuoteToken[], never, never>
  }
>() {}

// type FullQuoteToken = (QuoteToken, metadataType, metadata)
export type FullQuoteToken = [
  quoteToken: QuoteToken,
  metadataType: MetadataType,
  metadata: Metadata,
]

export type SourceChain = Chain & Brand.Brand<"SourceChain">
const SourceChain = Brand.nominal<SourceChain>()

export type DestinationChain = Chain & Brand.Brand<"DestinationChain">
const DestinationChain = Brand.nominal<DestinationChain>()

// req1 = SourceChain -> DestinationChain -> Channel[]
type Req1 = (
  source: SourceChain,
  destination: DestinationChain,
) => readonly Channel[]

// req2 = SourceChain -> Channel -> BaseToken -> FullQuoteToken[]
type Req2 = (
  source: SourceChain,
  channel: Channel,
  base: BaseToken,
) => readonly FullQuoteToken[]

// head FullQuoteToken[] -> FullQuoteToken
// TODO

// Chanel -> BaseToken -> FullQuoteToken -> FAOv2
declare const createFAOv2: (
  channel: Channel,
  base: BaseToken,
  quote: FullQuoteToken,
) => Effect.Effect<FAOv2>

/**
 * assuming canonical channel and canonical quote token
 */
// SourceChain -> DestinationChain -> BaseToken -> FAOv2
// Requirements = SourceWalletClient | GraphQL
export declare const _createFAOv2: (
  source: SourceChain,
  destination: DestinationChain,
  base: BaseToken,
) => Effect.Effect<FAOv2, never, WalletClient | GraphQL>

const babylon = SourceChain(
  new Chain({} as unknown as any, {}),
)
const bob = DestinationChain(
  new Chain({} as unknown as any, {}),
)
const baseToken = BaseToken.make(`0x123`)
const g = _createFAOv2(babylon, bob, baseToken)

/**
 * Three cases for req2
 */
// req2 = BaseToken -> Channel -> FullQuoteToken[]

/**
 * Case 1: Unwrapping
 * ---
 * If BaseToken is actually a wrapped representation from Channel, req2 will return the metadataType = unwrap
 */

/**
 * Case 2: Completely new asset
 * ---
 * OnchainMetadata { Name, Symbol, Decimals }
 * TBD
 */

/**
 * Case 3: preexisting wrapped asset
 * ---
 * we submit metadatatype = image and we provide for metadata the image.
 */
