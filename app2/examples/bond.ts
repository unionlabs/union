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
import { AddressCosmosZkgm } from "@unionlabs/sdk/schema/address"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import { HexFromJson, HexFromString } from "@unionlabs/sdk/schema/hex"
import { Bech32 } from "@unionlabs/sdk/Ucs05"
import { Effect, Logger, pipe, Schema } from "effect"
import {
  AbiParameter,
  bytesToHex,
  encodeAbiParameters,
  fromHex,
  http,
  keccak256,
  toHex,
} from "viem"
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

const VIEM_ACCOUNT = privateKeyToAccount(
  process.env.KEY as any,
)

const querySlippage = pipe(
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

const bytecode_base_checksum =
  "0xec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1" as const
const canonical_zkgm = Ucs05.anyDisplayToCanonical(UCS03_ZKGM)
const module_hash = "0x120970d812836f19888625587a4606a5ad23cef31c8684e601771552548fc6b9" as const

const instantiate2 = Effect.gen(function*() {
  const sender = yield* Ucs05.anyDisplayToZkgm(SENDER)
  const abi = [
    {
      name: "path",
      type: "uint256",
      internalType: "uint256",
    },
    {
      name: "channelId",
      type: "uint32",
      internalType: "uint32",
    },
    {
      name: "sender",
      type: "bytes",
      internalType: "bytes",
    },
  ] as const

  const args = [
    0n,
    20,
    sender,
  ] as const

  const encode = Effect.try(() =>
    encodeAbiParameters(
      abi,
      args,
    )
  )

  const encoded = yield* encode

  yield* Effect.log({ encoded })

  /**
   * n as be rep
   */
  const u64toBeBytes = (n: bigint) => {
    const buffer = new ArrayBuffer(8)
    const view = new DataView(buffer)
    view.setBigUint64(0, n)
    console.log(view.buffer)
    return view.buffer
  }

  const sha256 = (data: any) => globalThis.crypto.subtle.digest("SHA-256", data)

  const salt = keccak256(encoded, "bytes")

  const data = Uint8Array.from([
    fromHex(module_hash, "bytes"),
    "wasm",
    0, // null byte
    u64toBeBytes(32n), // checksum len as 64-bit big endian bytes of int
    fromHex(bytecode_base_checksum, "bytes"),
    u64toBeBytes(32n), // creator canonical addr len
    fromHex(canonical_zkgm, "bytes"),
    u64toBeBytes(32n), // len
    salt,
    u64toBeBytes(0n),
  ])

  const r = yield* Effect.tryPromise(() => sha256(data))

  const rBytes = bytesToHex(new Uint8Array(r))

  const transform = Ucs05.Bech32FromCanonicalBytesWithPrefix("union")

  const r2 = yield* Schema.decode(transform)(rBytes)

  yield* Effect.log("Salt:", bytesToHex(salt))
  yield* Effect.log("Args:", args)
  yield* Effect.log("Result:", r2)

  // yield* Effect.log(encoded)
})

const sendBond = Effect.gen(function*() {
  const ethereumChain = yield* ChainRegistry.byUniversalId(ETHEREUM_CHAIN_ID)
  const unionChain = yield* ChainRegistry.byUniversalId(UNION_CHAIN_ID)

  const tokenOrder = yield* TokenOrder.make({
    source: ethereumChain,
    destination: unionChain,
    sender: SENDER,
    receiver: ON_ZKGM_CALL_PROXY,
    baseToken: U_ERC20,
    baseAmount: SEND_AMOUNT,
    quoteToken: U_BANK,
    quoteAmount: SEND_AMOUNT,
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
        sender: SENDER,
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
        sender: SENDER,
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
        address: "union1ylfrhs2y5zdj2394m6fxgpzrjav7le3z07jffq",
      }),
      receiver: SENDER,
      baseToken: Token.Cw20.make({ address: EU_LST.address }),
      baseAmount: MIN_MINT_AMOUNT,
      quoteToken: EU_ERC20,
      quoteAmount: MIN_MINT_AMOUNT,
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

Effect.runPromise(instantiate2)
  .then(console.log)
  .catch(console.error)
