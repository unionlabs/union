import {
  Batch,
  Token,
  TokenOrder,
  Ucs03,
  Ucs05,
  ZkgmClient,
  ZkgmClientRequest,
  ZkgmClientResponse,
  ZkgmInstruction,
} from "@unionlabs/sdk"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import { ChannelRegistry } from "@unionlabs/sdk/ChannelRegistry"
// import { EvmClient } from "@unionlabs/sdk-evm"
import { EvmZkgmClient } from "@unionlabs/sdk-evm"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { Effect, pipe } from "effect"

// has a function .encode() -> ethabi (uses Ucs03 module)
// has a function .extractRequiredTokens() -> Token[]
//                                           example output: [{ token: Token.Erc20(`0x1234`), amount: 42342n }, { token: Token.EvmGas, amount: 200n }]

const program = Effect.gen(function*() {
  const source = yield* ChainRegistry.byUniversalId(
    UniversalChainId.make("bob.97"),
  )
  const destination = yield* ChainRegistry.byUniversalId(
    UniversalChainId.make("babylon.bbn-1"),
  )

  const incompleteTokenOrder = TokenOrder.make({
    source,
    destination,
    sender: Ucs05.EvmDisplay.make("0x123abcd"),
    receiver: Ucs05.CosmosDisplay.make("bbn1abcde"),
    baseToken: Token.Erc20.make({ address: "0x123" }),
    kind: TokenOrder.Kind.Escrow,
    baseAmount: 100n,
  })

  const batch: ZkgmInstruction.ZkgmInstruction = yield* pipe(
    incompleteTokenOrder,
    Effect.flatMap(TokenOrder.withAutoQuoteToken),
    Effect.flatMap(TokenOrder.withFee({ priority: "high" })),
  )

  const zkgmClient = yield* ZkgmClient.ZkgmClient

  const request = ZkgmClientRequest.make({
    source,
    destination,
    instruction: batch,
  })

  const response: ZkgmClientResponse.ZkgmClientResponse = yield* zkgmClient.execute(request)
}).pipe(
  EvmZkgmClient.fromBrowser({}),
  ChannelRegistry.Default,
  ChainRegistry.Default,
)
