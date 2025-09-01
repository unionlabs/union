/**
 * ETH -> UNION
 *
 * Execute from CLI with `KEY="0xprivatekey" pnpm dlx vite-node ./path/to/bond.ts`
 */
// @ts-ignore
if (typeof BigInt.prototype.toJSON !== "function") {
  // @ts-ignore
  BigInt.prototype.toJSON = function() {
    return this.toString()
  }
}
import {
  Batch,
  Call,
  TokenOrder,
  Ucs05,
  ZkgmClient,
  ZkgmClientRequest,
  ZkgmIncomingMessage,
} from "@unionlabs/sdk"
import { Evm, EvmZkgmClient } from "@unionlabs/sdk-evm"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import { ON_ZKGM_CALL_PROXY, U_BANK, U_ERC20, U_SOLVER_METADATA } from "@unionlabs/sdk/Constants"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import { HexFromJson } from "@unionlabs/sdk/schema/hex"
import { Effect, Logger, pipe, Schema } from "effect"
import { http } from "viem"
import { privateKeyToAccount } from "viem/accounts"
import { holesky } from "viem/chains"

const JsonFromBase64 = Schema.compose(
  Schema.StringFromBase64,
  Schema.parseJson(),
)

const AMOUNT = 1n
const ETHEREUM_CHAIN_ID = UniversalChainId.make("ethereum.17000")
const UNION_CHAIN_ID = UniversalChainId.make("union.union-testnet-10")
const SOURCE_CHANNEL_ID = ChannelId.make(1) // FIXME
const UCS03_MINTER = Ucs05.EvmDisplay.make({
  address: "0x5fbe74a283f7954f10aa04c2edf55578811aeb03",
})
const VIEM_CHAIN = holesky
const RPC_URL = "https://rpc.17000.ethereum.chain.kitchen"
const SENDER = Ucs05.EvmDisplay.make({
  address: "0x06627714f3F17a701f7074a12C02847a5D2Ca487",
})
const EU_STAKING_HUB = Ucs05.CosmosDisplay.make({
  address: "union1eueueueu9var4yhdruyzkjcsh74xzeug6ckyy60hs0vcqnzql2hq0lxc2f", // FIXME
})

const VIEM_ACCOUNT = privateKeyToAccount(
  process.env.KEY as any,
)

const sendUnbond = Effect.gen(function*() {
  const ethereumChain = yield* ChainRegistry.byUniversalId(ETHEREUM_CHAIN_ID)
  const unionChain = yield* ChainRegistry.byUniversalId(UNION_CHAIN_ID)

  const eu_staking_hub = yield* Ucs05.anyDisplayToZkgm(EU_STAKING_HUB)

  const tokenOrder = yield* TokenOrder.make({
    source: ethereumChain,
    destination: unionChain,
    sender: SENDER,
    receiver: ON_ZKGM_CALL_PROXY,
    baseToken: U_ERC20,
    baseAmount: AMOUNT,
    quoteToken: U_BANK,
    quoteAmount: AMOUNT,
    kind: "solve",
    metadata: U_SOLVER_METADATA,
    version: 2,
  })

  const unbondCall = yield* pipe(
    { amount: tokenOrder.quoteAmount } as const,
    Schema.encode(JsonFromBase64),
    Effect.map((msg) => ({
      contract: eu_staking_hub,
      msg,
      funds: [],
      call_action: "call_proxy",
    } as const)),
    Effect.flatMap(Schema.decode(HexFromJson)),
    Effect.map((contractCalldata) =>
      Call.make({
        sender: SENDER,
        eureka: false,
        contractAddress: ON_ZKGM_CALL_PROXY,
        contractCalldata,
      })
    ),
  )

  const batch = Batch.make([
    tokenOrder,
    unbondCall,
  ])

  const request = ZkgmClientRequest.make({
    source: ethereumChain,
    destination: unionChain,
    channelId: SOURCE_CHANNEL_ID,
    ucs03Address: UCS03_MINTER.address,
    instruction: batch,
  })

  const client = yield* ZkgmClient.ZkgmClient

  const response = yield* client.execute(request)
  yield* Effect.log("Submission TX Hash:", response.txHash)

  const receipt = yield* response.waitFor(
    ZkgmIncomingMessage.LifecycleEvent.$is("EvmTransactionReceiptComplete"),
  )

  yield* Effect.log("Receipt:", receipt)
}).pipe(
  Effect.provide(EvmZkgmClient.layerWithoutWallet),
  Effect.provide(Evm.WalletClient.Live({
    account: VIEM_ACCOUNT,
    chain: VIEM_CHAIN,
    transport: http(RPC_URL),
  })),
  Effect.provide(Evm.PublicClient.Live({
    chain: VIEM_CHAIN,
    transport: http(RPC_URL),
  })),
  Effect.provide(ChainRegistry.Default),
  Effect.provide(Logger.replace(
    Logger.defaultLogger,
    Logger.prettyLogger({
      stderr: true,
      colors: true,
      mode: "tty",
    }),
  )),
)

Effect.runPromise(sendUnbond)
  .then(console.log)
  .catch(console.error)
