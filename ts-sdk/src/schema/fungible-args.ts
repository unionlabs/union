import * as S from "effect/Schema"
import { Chain } from "./chain.js"
import { TokenRawAmount, TokenRawAmountFromSelf, TokenRawDenom } from "./token.js"
import { Effect, Match, ParseResult, Struct, pipe } from "effect"
import { AddressCanonicalBytes, AddressCosmosZkgm, AddressCosmosZkgmFromAddressCanonicalBytesWithPrefix, AddressEvmZkgm } from "./address.js"

export const BaseTransfer = S.Struct({
  sourceChain: Chain.annotations({
    message: () => "sourceChain cant be empty"
  }),
  destinationChain: Chain.annotations({
    message: () => "sourceChain cant be empty"
  }),
  sender: AddressCanonicalBytes,
  receiver: AddressCanonicalBytes,
  baseToken: S.NonEmptyString.pipe(
    S.annotations({
      message: () => "baseToken must be a non-empty string (e.g., token address or symbol)"
    }),
  ),
  baseAmount: TokenRawAmountFromSelf.annotations({
    message: () => "baseAmount must be a valid bigint string (e.g., \"1000000\")"
  }),
  quoteAmount: TokenRawAmountFromSelf.annotations({
    message: () => "quoteAmount must be a valid bigint string (e.g., \"1000000\")"
  }),
})
export type BaseTransfer = typeof BaseTransfer.Type

const EvmToEvm = S.Struct({
  ...BaseTransfer.fields,
  // wethBaseToken: EvmWethToken,
  receiver: AddressEvmZkgm,
  sender: AddressEvmZkgm,
  baseAmount: TokenRawAmountFromSelf,
  quoteAmount: TokenRawAmountFromSelf,
})
type EvmToEvm = typeof EvmToEvm.Type

const EvmToCosmos = S.Struct({
  ...BaseTransfer.fields,
  sender: AddressEvmZkgm,
  receiver: AddressCosmosZkgm,
  baseAmount: TokenRawAmountFromSelf,
  quoteAmount: TokenRawAmountFromSelf,
})
type EvmToCosmos = typeof EvmToCosmos.Type

const CosmosToEvm = S.Struct({
  ...BaseTransfer.fields,
  sender: AddressCosmosZkgm,
  receiver: AddressEvmZkgm,
  baseAmount: TokenRawAmountFromSelf,
  quoteAmount: TokenRawAmountFromSelf,
})
type CosmosToEvm = typeof CosmosToEvm.Type

const CosmosToCosmos = S.Struct({
  ...BaseTransfer.fields,
  sender: AddressCosmosZkgm,
  receiver: AddressCosmosZkgm,
  baseAmount: TokenRawAmountFromSelf,
  quoteAmount: TokenRawAmountFromSelf,
})
type CosmosToCosmos = typeof CosmosToCosmos.Type

const CosmosTransferSchema = S.Struct({
  ...BaseTransfer.fields,
  sourceRpcType: S.Literal("cosmos").annotations({
    message: () => "sourceRpcType must be 'cosmos'"
  }),
  receiver: S.String.pipe(
    S.nonEmptyString({ message: () => "receiver must be a non-empty string" })
  )
})
type CosmosTransferSchema = typeof CosmosTransferSchema.Type

export class CosmosTransfer extends S.Class<CosmosTransfer>("CosmosTransfer")(
  CosmosTransferSchema
) {}

const AptosTransferSchema = S.Struct({
  ...BaseTransfer.fields,
  sourceRpcType: S.Literal("aptos").annotations({
    message: () => "sourceRpcType must be 'aptos'"
  }),
  receiver: S.String.pipe(
    S.nonEmptyString({ message: () => "receiver must be a non-empty string" })
  )
})
type AptosTransferSchema = typeof AptosTransferSchema.Type

export class AptosTransfer extends S.Class<AptosTransfer>("AptosTransfer")(AptosTransferSchema) {}

export const FungibleIntent = S.Union(
  EvmToEvm,
  EvmToCosmos,
  CosmosToCosmos,
  CosmosToEvm,
).pipe(
  S.annotations({
    identifier: "Fungible Intent",
    description: "Discriminated fao arguments"
  })
)
export type FungibleIntent = typeof FungibleIntent.Type

export const AssetOrderIntentFromTransferIntent = S.transformOrFail(
  BaseTransfer,
  FungibleIntent,
  {
    strict: true,
    decode: (fromA) => {
      const matcher = Match.type<BaseTransfer>().pipe(
        Match.when(
          { 
            sourceChain: { rpc_type: "cosmos" },
            destinationChain: { rpc_type: "cosmos" }
          },
          (x) => S.decode(CosmosToCosmos)(Struct.evolve(x, {
            sender: S.decodeSync(
              AddressCosmosZkgmFromAddressCanonicalBytesWithPrefix(
                x.sourceChain.addr_prefix
              )
            ),
            receiver: S.decodeSync(AddressCosmosZkgmFromAddressCanonicalBytesWithPrefix(x.destinationChain.addr_prefix)),
          }))
        ),
        Match.when(
          { 
            sourceChain: { rpc_type: "cosmos" },
            destinationChain: { rpc_type: "evm" }
          },
          (x) => S.decode(CosmosToEvm)(Struct.evolve(x, {
            sender: S.decodeSync(
              AddressCosmosZkgmFromAddressCanonicalBytesWithPrefix(
                x.sourceChain.addr_prefix
              )
            ),
            receiver: AddressEvmZkgm.make
          }))
        ),
        Match.when(
          { 
            sourceChain: { rpc_type: "evm" },
            destinationChain: { rpc_type: "evm" }
          },
          (x) => S.decode(EvmToEvm)(Struct.evolve(x, {
            sender: AddressEvmZkgm.make,
            receiver: AddressEvmZkgm.make
          }))
        ),
        Match.when(
          { 
            sourceChain: { rpc_type: "evm" },
            destinationChain: { rpc_type: "cosmos" }
          },
          (x) => S.decode(EvmToCosmos)(Struct.evolve(x, {
            sender: AddressEvmZkgm.make,
            receiver: S.decodeSync(
              AddressCosmosZkgmFromAddressCanonicalBytesWithPrefix(
                x.destinationChain.addr_prefix
              )
            )
          }))
        ),
        Match.orElseAbsurd,
      )

      return pipe(
        matcher(fromA),
        Effect.mapError((x) => x.issue)
      )
    },
    encode: (toI, _, ast) => ParseResult.fail(
      new ParseResult.Forbidden(
        ast,
        toI,
        "Transforming from discriminated transfer to base transfer is not supported."
      )
    )
  }
)