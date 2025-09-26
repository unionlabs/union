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
  Ucs05,
  ZkgmClient,
  ZkgmClientRequest,
  ZkgmIncomingMessage,
} from "@unionlabs/sdk"
import { Evm, EvmZkgmClient } from "@unionlabs/sdk-evm"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import {
  EU_ERC20,
  EU_LST,
  EU_SOLVER_ON_UNION_METADATA,
  EU_STAKING_HUB,
} from "@unionlabs/sdk/Constants"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import { HexFromJson } from "@unionlabs/sdk/schema/hex"
import { Effect, Logger, pipe, Schema } from "effect"
import { bytesToHex, encodeAbiParameters, fromHex, http, keccak256 } from "viem"
import { privateKeyToAccount } from "viem/accounts"
import { holesky } from "viem/chains"

const JsonFromBase64 = Schema.compose(
  Schema.StringFromBase64,
  Schema.parseJson(),
)

const AMOUNT = 5n
const ETHEREUM_CHAIN_ID = UniversalChainId.make("ethereum.17000")
const UNION_CHAIN_ID = UniversalChainId.make("union.union-testnet-10")
const UCS03_ZKGM = Ucs05.CosmosDisplay.make({
  address: "union1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qpe64fh",
})
const SOURCE_CHANNEL_ID = ChannelId.make(6)
const DESTINATION_CHANNEL_ID = ChannelId.make(20)
const UCS03_EVM = Ucs05.EvmDisplay.make({
  address: "0x5fbe74a283f7954f10aa04c2edf55578811aeb03",
})
const VIEM_CHAIN = holesky
const RPC_URL = "https://rpc.17000.ethereum.chain.kitchen"
const VIEM_ACCOUNT = privateKeyToAccount(
  process.env.KEY as any,
)
const SENDER = Ucs05.EvmDisplay.make({
  address: VIEM_ACCOUNT.address,
})

const checkAndSubmitAllowance = pipe(
  Evm.readErc20Allowance(
    EU_ERC20.address,
    SENDER.address,
    UCS03_EVM.address,
  ),
  Effect.flatMap((amount) =>
    Effect.if(amount < AMOUNT, {
      onTrue: () =>
        pipe(
          Effect.log(`Increasing allowance by ${AMOUNT - amount} for ${EU_ERC20.address}`),
          Effect.andThen(() =>
            pipe(
              Evm.increaseErc20Allowance(
                EU_ERC20.address,
                UCS03_EVM,
                AMOUNT - amount,
              ),
              Effect.andThen(Evm.waitForTransactionReceipt),
            )
          ),
        ),
      onFalse: () =>
        Effect.log(`Allowance fulfilled by ${AMOUNT - amount} for ${EU_ERC20.address}`),
    })
  ),
)

const bytecode_base_checksum =
  "0xec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1" as const
const canonical_zkgm = Ucs05.anyDisplayToCanonical(UCS03_ZKGM)
const module_hash = "0x120970d812836f19888625587a4606a5ad23cef31c8684e601771552548fc6b9" as const

const instantiate2 = Effect.fn(
  function*(options: { path: bigint; channel: ChannelId; sender: Ucs05.AnyDisplay }) {
    const sender = yield* Ucs05.anyDisplayToZkgm(options.sender)
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
      options.path,
      options.channel,
      sender,
    ] as const

    const encode = Effect.try(() =>
      encodeAbiParameters(
        abi,
        args,
      )
    )

    const encoded = yield* encode

    /**
     * n as BE rep
     */
    const u64toBeBytes = (n: bigint) => {
      const buffer = new ArrayBuffer(8)
      const view = new DataView(buffer)
      view.setBigUint64(0, n)
      return new Uint8Array(view.buffer)
    }

    const sha256 = (data: any) => globalThis.crypto.subtle.digest("SHA-256", data)

    const salt = keccak256(encoded, "bytes")

    const _args = [
      ...fromHex(module_hash, "bytes"),
      ...new TextEncoder().encode("wasm"),
      0, // null byte
      ...u64toBeBytes(32n), // checksum len as 64-bit big endian bytes of int
      ...fromHex(bytecode_base_checksum, "bytes"),
      ...u64toBeBytes(32n), // creator canonical addr len
      ...fromHex(canonical_zkgm, "bytes"),
      ...u64toBeBytes(32n), // len
      ...salt,
      ...u64toBeBytes(0n),
    ] as const

    const data = Uint8Array.from(_args)

    const r = yield* Effect.tryPromise(() => sha256(data))

    const rBytes = bytesToHex(new Uint8Array(r))

    const transform = Ucs05.Bech32FromCanonicalBytesWithPrefix("union")

    const r2 = yield* Schema.decode(transform)(rBytes)

    return Ucs05.CosmosDisplay.make({ address: r2 })
  },
)

const sendUnbond = Effect.gen(function*() {
  const ethereumChain = yield* ChainRegistry.byUniversalId(ETHEREUM_CHAIN_ID)
  const unionChain = yield* ChainRegistry.byUniversalId(UNION_CHAIN_ID)
  const receiver = yield* instantiate2({
    path: 0n,
    channel: DESTINATION_CHANNEL_ID,
    sender: SENDER,
  })

  const tokenOrder = yield* TokenOrder.make({
    source: ethereumChain,
    destination: unionChain,
    sender: SENDER,
    receiver,
    baseToken: EU_ERC20,
    baseAmount: AMOUNT,
    quoteToken: Token.Cw20.make({ address: EU_LST.address }),
    quoteAmount: AMOUNT,
    kind: "solve",
    metadata: EU_SOLVER_ON_UNION_METADATA,
    version: 2,
  })

  const increaseAllowanceCall = yield* pipe(
    {
      increase_allowance: {
        spender: EU_STAKING_HUB.address,
        amount: AMOUNT,
      },
    } as const,
    Schema.encode(JsonFromBase64),
    Effect.map((msg) => ({
      wasm: {
        execute: {
          contract_addr: EU_LST.address,
          msg,
          funds: [],
        },
      },
    } as const)),
  )

  const unbondCall = yield* pipe(
    {
      unbond: {
        staker: receiver.address,
        amount: tokenOrder.quoteAmount,
      },
    } as const,
    Schema.encode(JsonFromBase64),
    Effect.map((msg) => ({
      wasm: {
        execute: {
          contract_addr: EU_STAKING_HUB.address,
          msg,
          funds: [],
        },
      },
    } as const)),
  )

  const calls = yield* pipe(
    [
      increaseAllowanceCall,
      unbondCall,
    ],
    Schema.decode(HexFromJson),
    Effect.map((contractCalldata) =>
      Call.make({
        sender: SENDER,
        eureka: false,
        contractAddress: receiver,
        contractCalldata,
      })
    ),
  )

  const batch = Batch.make([
    tokenOrder,
    calls,
  ])

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
})

pipe(
  Effect.all([
    checkAndSubmitAllowance,
    sendUnbond,
  ]),
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
  Effect.runPromise,
)
  .then(console.log)
  .catch(console.error)
