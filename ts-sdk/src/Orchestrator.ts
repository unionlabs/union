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

import * as Brand from "effect/Brand"
import * as S from "effect/Schema"
import { Chain } from "./schema/chain.js"

type FullQuoteToken = [quoteToken: QuoteToken, metadataType: MetadataType, metadata: Metadata]

export type SourceChain = Chain & Brand.Brand<"SourceChain">
export type DestinationChain = Chain & Brand.Brand<"DestinationChain">

type Req1 = 
