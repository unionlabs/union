import { Batch, Token, TokenOrder, Ucs03, Ucs05, ZkgmClient } from "@unionlabs/sdk"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import { ChannelRegistry } from "@unionlabs/sdk/ChannelRegistry"
// import { EvmClient } from "@unionlabs/sdk-evm"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { Arbitrary, Effect, Match, pipe } from "effect"

type ZkgmInstruction =
  // | Forward
  // | Call
  | Batch.Batch<ZkgmInstruction>
  | TokenOrder.TokenOrder

declare const a: ZkgmInstruction

const b = a._tag

const f = Match.type<ZkgmInstruction>().pipe(
  Match.tagsExhaustive({
    Batch: () => 0,
    TokenOrder: () => 0,
  }),
)

// has a function .encode() -> ethabi (uses Ucs03 module)
// has a function .extractRequiredTokens() -> Token[]
//                                           example output: [{ token: Token.Erc20(`0x1234`), amount: 42342n }, { token: Token.EvmGas, amount: 200n }]

type ZkgmRequest = {
  sourceChain: Chain<"evm">
  destinationChain: Chain<"cosmos">
  instruction: ZkgmInstruction
}

const program = Effect.gen(function*() {
  const sourceChain = yield* ChainRegistry.byUniversalId(
    UniversalChainId.make("bob.97"),
  )
  const destinationChain = yield* ChainRegistry.byUniversalId(
    UniversalChainId.make("babylon.bbn-1"),
  )

  const incompleteTokenOrder = // : PartialTokenOrder<"quoteToken" | "channel" | "quoteAmount"> =
    TokenOrder.make({
      sourceChain,
      destinationChain,
      sender: Ucs05.EvmDisplay.make("0x123abcd"),
      receiver: Ucs05.CosmosDisplay.make("bbn1abcde"),
      baseToken: Token.Erc20.make({ address: "0x123" }),
      type: TokenOrder.Type.Escrow,
      baseAmount: 100n,
    })

  const batch: ZkgmInstruction = yield* pipe(
    incompleteTokenOrder, // PartialTokenOrder<"quoteToken" | "channel">
    TokenOrder.withAutoQuoteToken, // (PartialTokenOrder<"quoteToken" | "channel"> -> Effect.Effect<PartialTokenOrder<"channel">, E, QuoteSerivce>
    Effect.flatMap(TokenOrder.withAutoChannel(sourceChain, destinationChain)),
    // (PartialTokenOrder<"channel"> -> Effect.Effect<PartialTokenOrder<never> = ZkgmInstruction.TokenOrder, E, ChannelSerivce>
    Effect.flatMap(TokenOrder.withFee({ priority: "high" })),
    // ZkgmInstruction.TokenOrder => Effect.Effect<ZkgmInstrunction.Batch([ZkgmInstrunction.TokenOrder, ZkgmInstrunction.TokenOrder]), E, FeeService>
  )

  const zkgmClient = yield* ZkgmClient.Client

  const request = {
    sourceChain,
    destinationChain,
    instruction: batch,
  } as const

  const response: ClientResponse = yield* zkgmClient.execute(batch)
}).pipe(
  ChannelRegistry.Default,
  ChainRegistry.Default,
)
