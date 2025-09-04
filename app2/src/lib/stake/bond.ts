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
import { Cosmos } from "@unionlabs/sdk-cosmos"
import { Evm, EvmZkgmClient } from "@unionlabs/sdk-evm"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import {
  EU_ERC20,
  EU_LST,
  EU_SOLVER_ON_UNION_METADATA,
  EU_STAKING_HUB,
  ON_ZKGM_CALL_PROXY,
  U_BANK,
  U_ERC20,
  U_SOLVER_ON_UNION_METADATA,
} from "@unionlabs/sdk/Constants"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import { HexFromJson } from "@unionlabs/sdk/schema/hex"
import { Effect, Logger, pipe, Schema, Stream } from "effect"
import { http } from "viem"
import { privateKeyToAccount } from "viem/accounts"
import { holesky } from "viem/chains"

const JsonFromBase64 = Schema.compose(
  Schema.StringFromBase64,
  Schema.parseJson(),
)

const ETHEREUM_CHAIN_ID = UniversalChainId.make("ethereum.17000")
const UNION_CHAIN_ID = UniversalChainId.make("union.union-testnet-10")
const SOURCE_CHANNEL_ID = ChannelId.make(6)
const UCS03_EVM = Ucs05.EvmDisplay.make({
  address: "0x5fbe74a283f7954f10aa04c2edf55578811aeb03",
})
const UCS03_MINTER_ON_UNION = Ucs05.CosmosDisplay.make({
  address: "union1t5awl707x54k6yyx7qfkuqp890dss2pqgwxh07cu44x5lrlvt4rs8hqmk0",
})
const UCS03_ZKGM = Ucs05.CosmosDisplay.make({
  address: "union1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qpe64fh",
})
const SEND_AMOUNT = (10n ** 18n) * 1_000n
const MIN_MINT_AMOUNT = 999999999000000000000n
const VIEM_CHAIN = holesky
const RPC_URL = "https://rpc.17000.ethereum.chain.kitchen"
const SENDER = Ucs05.EvmDisplay.make({
  address: "0x2C96e52fCE14BAa13868CA8182f8A7903e4e76E0",
})

export const querySlippage = pipe(
  Cosmos.queryContract(
    EU_STAKING_HUB,
    {
      accounting_state: {},
    },
  ),
  Effect.flatMap(Schema.decodeUnknown(Schema.Struct({
    total_bonded_native_tokens: Schema.BigInt,
    total_issued_lst: Schema.BigInt,
    total_reward_amount: Schema.BigInt,
    redemption_rate: Schema.BigDecimal,
    purchase_rate: Schema.BigDecimal,
  }))),
  Effect.provide(Cosmos.Client.Live("https://rpc.union-testnet-10.union.chain.kitchen")),
)

export const sendBond = Effect.fn(function*(options: {
  sender: Ucs05.EvmDisplay
  baseAmount: bigint
  quoteAmount: bigint
}) {
  const {
    sender,
    baseAmount,
    quoteAmount,
  } = options
  const ethereumChain = yield* ChainRegistry.byUniversalId(ETHEREUM_CHAIN_ID)
  const unionChain = yield* ChainRegistry.byUniversalId(UNION_CHAIN_ID)

  const tokenOrder = yield* TokenOrder.make({
    source: ethereumChain,
    destination: unionChain,
    sender,
    receiver: ON_ZKGM_CALL_PROXY,
    baseToken: U_ERC20,
    baseAmount,
    quoteToken: U_BANK,
    quoteAmount,
    kind: "solve",
    metadata: U_SOLVER_ON_UNION_METADATA,
    version: 2,
  })

  const bondCall = yield* pipe(
    {
      bond: {
        mint_to_address: ON_ZKGM_CALL_PROXY.address,
        min_mint_amount: MIN_MINT_AMOUNT,
      },
    } as const,
    Schema.encode(JsonFromBase64),
    Effect.map((msg) => ({
      contract: EU_STAKING_HUB.address,
      msg,
      funds: [{
        denom: tokenOrder.quoteToken.address,
        amount: tokenOrder.quoteAmount,
      }],
      call_action: "call_on_proxy_call",
    } as const)),
    Effect.flatMap(Schema.decode(HexFromJson)),
    Effect.map((contractCalldata) =>
      Call.make({
        sender,
        eureka: false,
        contractAddress: ON_ZKGM_CALL_PROXY,
        contractCalldata,
      })
    ),
  )

  const increaseAllowanceCall = yield* pipe(
    {
      increase_allowance: {
        spender: UCS03_MINTER_ON_UNION.address,
        amount: MIN_MINT_AMOUNT,
      },
    } as const,
    Schema.encode(JsonFromBase64),
    Effect.map((msg) => ({
      contract: EU_LST.address,
      msg,
      funds: [],
      call_action: "direct",
    } as const)),
    Effect.flatMap(Schema.decode(HexFromJson)),
    Effect.map((contractCalldata) =>
      Call.make({
        sender,
        eureka: false,
        contractAddress: ON_ZKGM_CALL_PROXY,
        contractCalldata,
      })
    ),
  )

  const salt = yield* Utils.generateSalt("cosmos")
  const timeout_timestamp = Utils.getTimeoutInNanoseconds24HoursFromNow()

  const sendCall = yield* pipe(
    TokenOrder.make({
      source: unionChain,
      destination: ethereumChain,
      sender: Ucs05.CosmosDisplay.make({
        address: "union1ylfrhs2y5zdj2394m6fxgpzrjav7le3z07jffq", // NOTE: foundation for refunds
      }),
      receiver: sender,
      baseToken: Token.Cw20.make({ address: EU_LST.address }),
      baseAmount,
      quoteToken: EU_ERC20,
      quoteAmount,
      kind: "solve",
      metadata: EU_SOLVER_ON_UNION_METADATA,
      version: 2,
    }),
    Effect.flatMap(TokenOrder.encodeV2),
    Effect.flatMap(Schema.encode(Ucs03.Ucs03WithInstructionFromHex)),
    Effect.tap((instr) => Effect.log("instruction:", instr)),
    Effect.map((instruction) => ({
      send: {
        channel_id: 20,
        timeout_height: 0n,
        timeout_timestamp,
        salt,
        instruction,
      },
    } as const)),
    Effect.flatMap(Schema.encode(JsonFromBase64)),
    Effect.map((msg) => ({
      contract: UCS03_ZKGM.address,
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

  console.log("batch", JSON.stringify(batch, null, 2))

  const request = ZkgmClientRequest.make({
    source: ethereumChain,
    destination: unionChain,
    channelId: SOURCE_CHANNEL_ID,
    ucs03Address: UCS03_EVM.address,
    instruction: batch,
  })

  const client = yield* ZkgmClient.ZkgmClient
  const execute = client.execute(request)

  return pipe(
    Stream.fromEffect(execute),
    Stream.flatMap((response) =>
      response.waitFor(
        ZkgmIncomingMessage.LifecycleEvent.$is("EvmTransactionReceiptComplete"),
      )
    ),
  )
})
// }).pipe(
//   Effect.provide(EvmZkgmClient.layerWithoutWallet),
//   Effect.provide(Evm.WalletClient.Live({
//     account: VIEM_ACCOUNT,
//     chain: VIEM_CHAIN,
//     transport: http(RPC_URL),
//   })),
//   Effect.provide(Evm.PublicClient.Live({
//     chain: VIEM_CHAIN,
//     transport: http(RPC_URL),
//   })),
//   Effect.provide(ChainRegistry.Default),
//   Effect.provide(Logger.replace(
//     Logger.defaultLogger,
//     Logger.prettyLogger({
//       stderr: true,
//       colors: true,
//       mode: "tty",
//     }),
//   )),
// )

// Effect.runPromise(sendBond)
//   .then(console.log)
//   .catch(console.error)
