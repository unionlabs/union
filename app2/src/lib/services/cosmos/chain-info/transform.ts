import { Effect, flow, ParseResult, pipe, Schema as S, Struct, Tuple as T } from "effect"
import type { NoExcessProperties } from "effect/Types"
import { InternalChainInfo } from "./internal-chain-info"
import { ChainInfo as KeplrChainInfo } from "./keplr-chain-info.js"
import { ChainInfo as LeapChainInfo } from "./leap-chain-info.js"

export const KeplrChainInfoFromInternal = S.transformOrFail(
  InternalChainInfo,
  S.Tuple(KeplrChainInfo, InternalChainInfo),
  {
    decode: (fromA) =>
      Effect.gen(function*() {
        const _ = pipe(
          fromA,
          Struct.omit("theme"),
          Struct.omit("image"),
        )
        return [
          _ satisfies NoExcessProperties<KeplrChainInfo, typeof _>,
          fromA,
        ] as const
      }),
    encode: flow(T.getSecond, InternalChainInfo.make, ParseResult.succeed),
    strict: true,
  },
)

export const LeapChainInfoFromInternal = S.transformOrFail(
  InternalChainInfo,
  S.Tuple(LeapChainInfo, InternalChainInfo),
  {
    decode: (fromA) =>
      Effect.gen(function*() {
        const _ = pipe(
          fromA,
          Struct.omit("nodeProvider"),
          Struct.omit("alternativeBIP44s"),
          Struct.omit("chainSymbolImageUrl"),
          Struct.omit("hideInUI"),
          Struct.omit("evm"),
        )
        return [
          _ satisfies NoExcessProperties<LeapChainInfo, typeof _>,
          fromA,
        ] as const
      }),
    encode: flow(T.getSecond, InternalChainInfo.make, ParseResult.succeed),
    strict: true,
  },
)
