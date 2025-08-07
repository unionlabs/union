import {
  Token,
  TokenOrder,
  Ucs05,
  ZkgmClient,
  ZkgmClientRequest,
  ZkgmClientResponse,
  ZkgmIncomingMessage,
} from "@unionlabs/sdk"
import { EvmWallet, EvmZkgmClient } from "@unionlabs/sdk-evm"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
// import { ChannelRegistry } from "@unionlabs/sdk/ChannelRegistry"
// import { FeeEstimator } from "@unionlabs/sdk/FeeEstimator"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
// import { TokenRegistry } from "@unionlabs/sdk/TokenRegistry"
import { Effect, Logger, pipe } from "effect"
import { http } from "viem"
import { privateKeyToAccount } from "viem/accounts"
import { holesky } from "viem/chains"

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

  const tokenOrder = yield* TokenOrder.make({
    source,
    destination,
    sender: "0x06627714f3F17a701f7074a12C02847a5D2Ca487",
    receiver: "0x06627714f3F17a701f7074a12C02847a5D2Ca487",
    baseToken: "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
    baseAmount: 100n,
    quoteToken: "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
    quoteAmount: 100n,
    kind: TokenOrder.Kind.Escrow,
    metadata: undefined,
  })

  const request = ZkgmClientRequest.make({
    source,
    destination,
    channelId: ChannelId.make(2),
    ucs03Address: "0x5fbe74a283f7954f10aa04c2edf55578811aeb03",
    instruction: tokenOrder,
  })

  const zkgmClient = yield* ZkgmClient.ZkgmClient

  // const response: ZkgmClientResponse.ZkgmClientResponse = yield* zkgmClient.execute(request)

  // const completion = yield* response.waitFor(ZkgmIncomingMessage.isComplete)

  // yield* Effect.log(completion.txHash)
}).pipe(
  Effect.provide(EvmZkgmClient.layerWithoutWallet),
  Effect.provide(EvmWallet.EvmWallet.Live({
    account: privateKeyToAccount("0x..."),
    chain: holesky,
    transport: http("https://rpc.17000.holesky.chain.kitchen"),
  })),
  // Effect.provide(ChannelRegistry.Default),
  // Effect.provide(FeeEstimator.Default),
  // Effect.provide(TokenRegistry.Default),
  Effect.provide(ChainRegistry.Default),
  Effect.provide(Logger.replace(Logger.defaultLogger, Logger.prettyLoggerDefault)),
)

Effect.runPromise(program)
  .then(console.log)
  .catch(console.error)
