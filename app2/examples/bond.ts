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
//this are important imports 
import {
  Batch,
  Call,
  Token,
  TokenOrder,
  Ucs03,
  Ucs05,
  Utils,
  ZkgmClient,
  ZkgmClientRequest,
  ZkgmIncomingMessage,
} from "@unionlabs/sdk"
import { Evm, EvmZkgmClient } from "@unionlabs/sdk-evm"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import {
  EU_ERC20,
  EU_FROM_UNION_SOLVER_METADATA,
  EU_LST,
  ON_ZKGM_CALL_PROXY,
  U_BANK,
  U_ERC20,
  U_TO_UNION_SOLVER_METADATA,
} from "@unionlabs/sdk/Constants"
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

const ETHEREUM_CHAIN_ID = UniversalChainId.make("ethereum.17000")
const UNION_CHAIN_ID = UniversalChainId.make("union.union-testnet-10")
const SOURCE_CHANNEL_ID = ChannelId.make(1) // FIXME
const UCS03_MINTER = Ucs05.EvmDisplay.make({
  address: "0x5fbe74a283f7954f10aa04c2edf55578811aeb03",
})
const MIN_MINT_AMOUNT = 1n
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

const sendBond = Effect.gen(function*() {
  const ethereumChain = yield* ChainRegistry.byUniversalId(ETHEREUM_CHAIN_ID)
  const unionChain = yield* ChainRegistry.byUniversalId(UNION_CHAIN_ID)

  const eu_staking_hub = yield* Ucs05.anyDisplayToZkgm(EU_STAKING_HUB)
  const on_zkgm_call_proxy = yield* Ucs05.anyDisplayToZkgm(ON_ZKGM_CALL_PROXY)
  const ucs03_minter = yield* Ucs05.anyDisplayToZkgm(UCS03_MINTER)
  const eu_lst = yield* Ucs05.anyDisplayToZkgm(EU_LST)

  const tokenOrder = yield* TokenOrder.make({
    source: ethereumChain,
    destination: unionChain,
    sender: SENDER,
    receiver: ON_ZKGM_CALL_PROXY,
    baseToken: U_ERC20,
    baseAmount: 1n,
    quoteToken: U_BANK,
    quoteAmount: 1n,
    kind: "solve",
    metadata: U_TO_UNION_SOLVER_METADATA,
    version: 2,
  })

  const bondCall = yield* pipe(
    {
      mint_to: on_zkgm_call_proxy,
      min_mint_amount: MIN_MINT_AMOUNT,
    } as const,
    Schema.encode(JsonFromBase64),
    Effect.map((msg) => ({
      contract: eu_staking_hub,
      msg,
      funds: [{
        denom: tokenOrder.quoteToken.address,
        amount: tokenOrder.quoteAmount,
      }],
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

  const increaseAllowanceCall = yield* pipe(
    {
      spender: ucs03_minter,
      amount: MIN_MINT_AMOUNT,
    } as const,
    Schema.encode(JsonFromBase64),
    Effect.map((msg) => ({
      contract: eu_lst,
      msg,
      funds: [],
      call_action: "direct",
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

  const salt = yield* Utils.generateSalt("cosmos")

  const sendCall = yield* pipe(
    TokenOrder.make({
      source: unionChain,
      destination: ethereumChain,
      sender: ON_ZKGM_CALL_PROXY, // FIXME: foundation multisig
      receiver: SENDER,
      baseToken: Token.Cw20.make({ address: EU_LST.address }),
      baseAmount: MIN_MINT_AMOUNT,
      quoteToken: EU_ERC20,
      quoteAmount: MIN_MINT_AMOUNT,
      kind: "solve",
      metadata: EU_FROM_UNION_SOLVER_METADATA,
      version: 2,
    }),
    Effect.flatMap(TokenOrder.encodeV2),
    Effect.flatMap(Schema.encode(Ucs03.Ucs03WithInstructionFromHex)),
    Effect.tap((instr) => Effect.log("instr:", instr)),
    Effect.map((instruction) => ({
      path: 0n,
      channel_id: 19,
      salt,
      instruction,
    } as const)),
    Effect.flatMap(Schema.encode(JsonFromBase64)),
    Effect.map((msg) => ({
      contract: ucs03_minter,
      msg,
      funds: [],
      call_action: "direct",
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
    bondCall,
    increaseAllowanceCall,
    sendCall,
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

Effect.runPromise(sendBond)
  .then(console.log)
  .catch(console.error)
