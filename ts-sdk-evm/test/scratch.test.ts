import {
  Token,
  TokenOrder,
  Ucs05,
  ZkgmClient,
  ZkgmClientRequest,
  ZkgmClientResponse,
  ZkgmIncomingMessage,
} from "@unionlabs/sdk"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import { ChannelRegistry } from "@unionlabs/sdk/ChannelRegistry"
import { FeeEstimator } from "@unionlabs/sdk/FeeEstimator"
import { TokenRegistry } from "@unionlabs/sdk/TokenRegistry"
// import { EvmClient } from "@unionlabs/sdk-evm"
import { EvmZkgmClient } from "@unionlabs/sdk-evm"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { Effect, Logger, pipe } from "effect"

// has a function .encode() -> ethabi (uses Ucs03 module)
// has a function .extractRequiredTokens() -> Token[]
//                                           example output: [{ token: Token.Erc20(`0x1234`), amount: 42342n }, { token: Token.EvmGas, amount: 200n }]

const program = Effect.gen(function*() {
  const source = yield* ChainRegistry.byUniversalId(
    UniversalChainId.make("ethereum.11155111"),
  )
  const destination = yield* ChainRegistry.byUniversalId(
    UniversalChainId.make("ethereum.17000"),
  )

  const incompleteTokenOrder = TokenOrder.make({
    source,
    destination,
    sender: "0x06627714f3F17a701f7074a12C02847a5D2Ca487",
    receiver: Ucs05.EvmDisplay.make("0x06627714f3F17a701f7074a12C02847a5D2Ca487", {
      disableValidation: true,
    }),
    baseToken: "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
    baseAmount: 100n,
    kind: TokenOrder.Kind.Escrow,
    quoteAmount: 50n,
    metadata: undefined,
  })

  yield* Effect.log({
    incompleteTokenOrder: yield* incompleteTokenOrder,
  })

  const tokenOrder = yield* pipe(
    incompleteTokenOrder,
    Effect.tap((incomplete) => Effect.log({ incomplete })),
    Effect.flatMap(TokenOrder.withAutoQuoteToken),
  )

  // const zkgmClient = yield* ZkgmClient.ZkgmClient

  // const request = ZkgmClientRequest.make({
  //   source,
  //   destination,
  //   instruction: tokenOrder,
  // })

  // const response: ZkgmClientResponse.ZkgmClientResponse = yield* zkgmClient.execute(request)

  // const completion = yield* response.waitFor(ZkgmIncomingMessage.isComplete)

  // yield* Effect.log(completion.txHash)
}).pipe(
  // Effect.provide(EvmZkgmClient.layerWithoutWallet),
  // Effect.provide(ChannelRegistry.Default),
  // Effect.provide(FeeEstimator.Default),
  // Effect.provide(TokenRegistry.Default),
  Effect.provide(ChainRegistry.Default),
  Effect.provide(Logger.replace(Logger.defaultLogger, Logger.prettyLoggerDefault)),
)

Effect.runPromise(program)
  .then(console.log)
  .catch(console.error)
