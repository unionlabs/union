<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Input from "$lib/components/ui/Input.svelte"
import Label from "$lib/components/ui/Label.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { runPromiseExit$ } from "$lib/runtime"
import { getWagmiConnectorClient } from "$lib/services/evm/clients"
import { switchChain } from "$lib/services/transfer-ucs03-evm/chain"
import { wallets as WalletStore } from "$lib/stores/wallets.svelte"
import { safeOpts } from "$lib/transfer/shared/services/handlers/safe"
import { cn } from "$lib/utils"
import { matchOption } from "$lib/utils/snippets.svelte"
import { getLastConnectedWalletId } from "$lib/wallet/evm/config.svelte"
import {
  Batch,
  Call,
  Indexer,
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
import { Evm, EvmZkgmClient, Safe } from "@unionlabs/sdk-evm"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import {
  EU_ERC20,
  EU_LST,
  EU_SOLVER_ON_ETH_METADATA,
  EU_STAKING_HUB,
  U_BANK,
  U_ERC20,
  U_SOLVER_ON_UNION_METADATA,
} from "@unionlabs/sdk/Constants"
import type { Chain, Token as TokenType } from "@unionlabs/sdk/schema"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import { HexFromJson } from "@unionlabs/sdk/schema/hex"
import {
  BigDecimal,
  ConfigProvider,
  Data,
  Effect,
  Layer,
  Match,
  pipe,
  Schedule,
  Schema,
  Struct,
} from "effect"
import * as A from "effect/Array"
import * as O from "effect/Option"
import { graphql } from "gql.tada"
import { custom } from "viem"
import { bytesToHex, encodeAbiParameters, fromHex, keccak256 } from "viem"
import { sepolia } from "viem/chains"

const ETHEREUM_CHAIN_ID = UniversalChainId.make("ethereum.11155111")
const UNION_CHAIN_ID = UniversalChainId.make("union.union-testnet-10")
const SOURCE_CHANNEL_ID = ChannelId.make(3)
const DESTINATION_CHANNEL_ID = ChannelId.make(3)
const UCS03_EVM = Ucs05.EvmDisplay.make({
  address: "0x5fbe74a283f7954f10aa04c2edf55578811aeb03",
})
const UCS03_MINTER_ON_UNION = Ucs05.CosmosDisplay.make({
  address: "union1t5awl707x54k6yyx7qfkuqp890dss2pqgwxh07cu44x5lrlvt4rs8hqmk0",
})
const UCS03_ZKGM = Ucs05.CosmosDisplay.make({
  address: "union1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qpe64fh",
})

const JsonFromBase64 = Schema.compose(
  Schema.StringFromBase64,
  Schema.parseJson(),
)

interface Props {
  evmChain: O.Option<Chain>
  uOnEvmToken: O.Option<TokenType>
  uOnEvmBalance: O.Option<bigint>
  onBondSuccess?: () => void
}

let { evmChain, uOnEvmToken, uOnEvmBalance, onBondSuccess }: Props = $props()

type BondState = Data.TaggedEnum<{
  Ready: {}
  SwitchingChain: {}
  CheckingAllowance: {}
  ApprovingAllowance: {}
  AllowanceSubmitted: { txHash: string }
  WaitingForAllowanceConfirmation: { txHash: string }
  AllowanceApproved: {}
  CreatingTokenOrder: {}
  PreparingBondTransaction: {}
  ConfirmingBond: {}
  BondSubmitted: { txHash: string }
  WaitingForConfirmation: { txHash: string }
  WaitingForIndexer: { txHash: string }
  Success: { txHash: string }
  Error: { message: string }
}>

const BondState = Data.taggedEnum<BondState>()

let bondInput = $state<string>("")
let bondState = $state<BondState>(BondState.Ready())
let shouldBond = $state<boolean>(false)
let slippage = $state<number>(1)

const stakingRates = runPromiseExit$(() =>
  Effect.gen(function*() {
    return yield* pipe(
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
  })
)

const isBonding = $derived(
  !BondState.$is("Ready")(bondState)
    && !BondState.$is("Success")(bondState)
    && !BondState.$is("Error")(bondState),
)
const isSuccess = $derived(BondState.$is("Success")(bondState))
const isError = $derived(BondState.$is("Error")(bondState))

const bytecode_base_checksum =
  "0xec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1" as const
const canonical_zkgm = Ucs05.anyDisplayToCanonical(UCS03_ZKGM)
const module_hash = "0x120970d812836f19888625587a4606a5ad23cef31c8684e601771552548fc6b9" as const

const QlpConfigProvider = pipe(
  ConfigProvider.fromMap(
    new Map([
      ["GRAPHQL_ENDPOINT", "https://development.graphql.union.build/v1/graphql"],
    ]),
  ),
  Layer.setConfigProvider,
)

const inputAmount = $derived<O.Option<BigDecimal.BigDecimal>>(pipe(
  bondInput,
  BigDecimal.fromString,
))

const bondAmount = $derived<O.Option<bigint>>(pipe(
  inputAmount,
  O.map(bd => {
    const result = BigDecimal.multiply(bd, BigDecimal.make(10n ** 18n, 0))
    const normalized = BigDecimal.normalize(result)
    return normalized.scale >= 0 
      ? normalized.value / (10n ** BigInt(normalized.scale))
      : normalized.value * (10n ** BigInt(-normalized.scale))
  }),
))

const minimumReceiveAmount = $derived<O.Option<BigDecimal.BigDecimal>>(pipe(
  O.Do,
  O.bind("input", () => inputAmount),
  O.bind(
    "rates",
    () =>
      O.isSome(stakingRates.current) && stakingRates.current.value._tag === "Success"
        ? O.some(stakingRates.current.value.value)
        : O.none(),
  ),
  O.map(({ input, rates }) => {
    const inputNorm = BigDecimal.normalize(input)
    const rateNorm = BigDecimal.normalize(rates.purchase_rate)

    const expectedScaled = inputNorm.value * rateNorm.value
    const minScaled = expectedScaled * BigInt(100 - slippage) / 100n
    return BigDecimal.make(minScaled, inputNorm.scale + rateNorm.scale)
  }),
))

const instantiate2 = Effect.fn(
  function*(options: { path: bigint; channel: ChannelId; sender: Ucs05.EvmDisplay }) {
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

    const salt = yield* pipe(
      Effect.try(() =>
        encodeAbiParameters(
          abi,
          [
            options.path,
            options.channel,
            sender,
          ] as const,
        )
      ),
      Effect.map((encoded) => keccak256(encoded, "bytes")),
    )

    const u64toBeBytes = (n: bigint) => {
      const buffer = new ArrayBuffer(8)
      const view = new DataView(buffer)
      view.setBigUint64(0, n)
      return new Uint8Array(view.buffer)
    }

    const sha256 = Effect.fn((data: any) =>
      Effect.tryPromise(() => globalThis.crypto.subtle.digest("SHA-256", data))
    )

    const address = yield* pipe(
      Uint8Array.from(
        [
          ...fromHex(module_hash, "bytes"),
          ...new TextEncoder().encode("wasm"),
          0,
          ...u64toBeBytes(32n),
          ...fromHex(bytecode_base_checksum, "bytes"),
          ...u64toBeBytes(32n),
          ...fromHex(canonical_zkgm, "bytes"),
          ...u64toBeBytes(32n),
          ...salt,
          ...u64toBeBytes(0n),
        ],
      ),
      sha256,
      Effect.map((r) => new Uint8Array(r)),
      Effect.map(bytesToHex),
      Effect.flatMap(
        Schema.decode(Ucs05.Bech32FromCanonicalBytesWithPrefix("union")),
      ),
    )

    return Ucs05.CosmosDisplay.make({ address })
  },
)

const checkAndSubmitAllowance = (sender: Ucs05.EvmDisplay, sendAmount: bigint) =>
  Effect.gen(function*() {
    bondState = BondState.CheckingAllowance()

    const currentAllowance = yield* Evm.readErc20Allowance(
      U_ERC20.address,
      sender.address,
      UCS03_EVM.address,
    )

    if (currentAllowance < sendAmount) {
      bondState = BondState.ApprovingAllowance()

      const approveTxHash = yield* Evm.increaseErc20Allowance(
        U_ERC20.address,
        UCS03_EVM,
        sendAmount,
      )

      bondState = BondState.AllowanceSubmitted({ txHash: approveTxHash })
      yield* Effect.sleep("500 millis")

      bondState = BondState.WaitingForAllowanceConfirmation({ txHash: approveTxHash })
      yield* Evm.waitForTransactionReceipt(approveTxHash)
    }

    bondState = BondState.AllowanceApproved()
    yield* Effect.sleep("500 millis")
  })

const executeBond = (sender: Ucs05.EvmDisplay, sendAmount: bigint, slippagePercent: number) =>
  Effect.gen(function*() {
    const minMintAmount = sendAmount * BigInt(100 - slippagePercent) / 100n

    const ethereumChain = yield* ChainRegistry.byUniversalId(ETHEREUM_CHAIN_ID)
    const unionChain = yield* ChainRegistry.byUniversalId(UNION_CHAIN_ID)
    const receiver = yield* instantiate2({
      path: 0n,
      channel: DESTINATION_CHANNEL_ID,
      sender,
    })

    const tokenOrder = yield* TokenOrder.make({
      source: ethereumChain,
      destination: unionChain,
      sender,
      receiver,
      baseToken: U_ERC20,
      baseAmount: sendAmount,
      quoteToken: U_BANK,
      quoteAmount: sendAmount,
      kind: "solve",
      metadata: U_SOLVER_ON_UNION_METADATA,
      version: 2,
    })

    const bondCall = yield* pipe(
      {
        bond: {
          mint_to_address: receiver.address,
          min_mint_amount: minMintAmount,
        },
      } as const,
      Schema.encode(JsonFromBase64),
      Effect.map((msg) => ({
        wasm: {
          execute: {
            contract_addr: EU_STAKING_HUB.address,
            msg,
            funds: [
              { denom: U_BANK.address, amount: sendAmount },
            ],
          },
        },
      })),
    )

    const increaseAllowanceCall = yield* pipe(
      {
        increase_allowance: {
          spender: UCS03_MINTER_ON_UNION.address,
          amount: minMintAmount,
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
      })),
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
        receiver: sender,
        baseToken: Token.Cw20.make({ address: EU_LST.address }),
        baseAmount: minMintAmount,
        quoteToken: EU_ERC20,
        quoteAmount: minMintAmount,
        kind: "solve",
        metadata: EU_SOLVER_ON_ETH_METADATA,
        version: 2,
      }),
      Effect.flatMap(TokenOrder.encodeV2),
      Effect.flatMap(Schema.encode(Ucs03.Ucs03WithInstructionFromHex)),
      Effect.map((instruction) => ({
        send: {
          channel_id: DESTINATION_CHANNEL_ID,
          timeout_height: 0n,
          timeout_timestamp,
          salt,
          instruction,
        },
      } as const)),
      Effect.flatMap(Schema.encode(JsonFromBase64)),
      Effect.map((msg) => ({
        wasm: {
          execute: {
            contract_addr: UCS03_ZKGM.address,
            msg,
            funds: [],
          },
        },
      })),
    )

    const calls = yield* pipe(
      [
        bondCall,
        increaseAllowanceCall,
        sendCall,
      ],
      Schema.decode(HexFromJson),
      Effect.map((contractCalldata) =>
        Call.make({
          sender,
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
    return yield* client.execute(request)
  })

runPromiseExit$(() =>
  shouldBond
    ? Effect.gen(function*() {
      const senderOpt = WalletStore.evmAddress
      if (O.isNone(senderOpt) || O.isNone(bondAmount) || O.isNone(evmChain)) {
        bondState = BondState.Error({
          message: "Missing required data: wallet address, bond amount, or chain",
        })
        shouldBond = false
        return yield* Effect.fail(new Error("Missing required data"))
      }

      const sender = senderOpt.value
      const sendAmount = O.getOrThrow(bondAmount)
      const chain = evmChain.value

      bondState = BondState.SwitchingChain()

      const VIEM_CHAIN = sepolia

      const connectorClient = yield* getWagmiConnectorClient

      const isSafeWallet = getLastConnectedWalletId() === "safe"

      if (!isSafeWallet) {
        yield* switchChain(VIEM_CHAIN)
      }

      const publicClient = Evm.PublicClient.Live({
        chain: VIEM_CHAIN,
        transport: custom(connectorClient),
      })

      const walletClient = Evm.WalletClient.Live({
        account: connectorClient.account,
        chain: VIEM_CHAIN,
        transport: custom(connectorClient),
      })

      yield* checkAndSubmitAllowance(sender, sendAmount).pipe(
        Effect.provide(walletClient),
        Effect.provide(publicClient),
      )

      bondState = BondState.CreatingTokenOrder()
      yield* Effect.sleep("300 millis")

      bondState = BondState.PreparingBondTransaction()
      yield* Effect.sleep("300 millis")

      bondState = BondState.ConfirmingBond()

      const executeBondWithProviders = isSafeWallet
        ? executeBond(sender, sendAmount, slippage).pipe(
          Effect.provide(EvmZkgmClient.layerWithoutWallet),
          Effect.provide(walletClient),
          Effect.provide(publicClient),
          Effect.provide(ChainRegistry.Default),
          Effect.provide(Safe.Safe.Default({
            ...safeOpts,
            debug: true,
          })),
        )
        : executeBond(sender, sendAmount, slippage).pipe(
          Effect.provide(EvmZkgmClient.layerWithoutWallet),
          Effect.provide(walletClient),
          Effect.provide(publicClient),
          Effect.provide(ChainRegistry.Default),
        )

      const response = yield* executeBondWithProviders
      const txHash = response.txHash

      bondState = BondState.BondSubmitted({ txHash })
      yield* Effect.sleep("500 millis")

      bondState = BondState.WaitingForConfirmation({ txHash })

      const finalHash = yield* Effect.if(isSafeWallet, {
        onTrue: () =>
          pipe(
            response.waitFor(
              ZkgmIncomingMessage.LifecycleEvent.$is("WaitForSafeWalletHash"),
            ),
            Effect.flatMap(O.map(x => x.hash)),
          ),
        onFalse: () =>
          pipe(
            response.waitFor(
              ZkgmIncomingMessage.LifecycleEvent.$is("EvmTransactionReceiptComplete"),
            ),
            Effect.flatMap(O.map(x => x.transactionHash)),
          ),
      })

      bondState = BondState.WaitingForIndexer({ txHash: finalHash })

      yield* pipe(
        Effect.gen(function*() {
          const indexer = yield* Indexer.Indexer
          return yield* indexer.fetch({
            document: graphql(`
               query GetBondByTxHash($tx_hash: String!) @cached(ttl: 10) {
                 v2_bonds(args: { p_transaction_hash: $tx_hash }) {
                   packet_hash
                 }
               }
             `),
            variables: { tx_hash: finalHash },
          })
        }),
        Effect.flatMap(Schema.decodeUnknown(
          Schema.Struct({
            v2_bonds: Schema.NonEmptyArray(Schema.Struct({ packet_hash: Schema.String })),
          }),
        )),
        Effect.retry({
          schedule: Schedule.fixed("2 seconds"),
          times: 30,
          while: (error) => String(error.message || "").includes("is missing"),
        }),
        Effect.provide(Indexer.Indexer.Default),
        Effect.provide(QlpConfigProvider),
      )

      bondState = BondState.Success({ txHash: finalHash })

      bondInput = ""
      shouldBond = false
      onBondSuccess?.()
    }).pipe(
      Effect.catchAll(error =>
        Effect.gen(function*() {
          const errorObj = error as any
          const fullError = errorObj?.cause?.cause?.shortMessage
            || errorObj?.cause?.message
            || errorObj?.message
            || JSON.stringify(error)
          const shortMessage = String(fullError).split(".")[0]

          bondState = BondState.Error({ message: shortMessage })
          shouldBond = false
          return yield* Effect.succeed(false)
        })
      ),
    )
    : Effect.void
)

function handleBondSubmit() {
  if (isBonding) {
    return
  }
  bondState = BondState.Ready()
  shouldBond = true
}

function handleRetry() {
  bondState = BondState.Ready()
}
</script>

{#snippet renderBalanceSkeleton()}
  <Skeleton class="w-full h-6 ml-auto" />
{/snippet}

{#snippet renderBalance(amount: bigint)}
  <div class="font-mono">
    {
      pipe(
        BigDecimal.fromBigInt(amount),
        BigDecimal.unsafeDivide(BigDecimal.make(1n, -18)),
        Utils.formatBigDecimal,
      )
    }
  </div>
{/snippet}

<div class="flex grow flex-col gap-4">
  <div>
    <Label caseSensitive>U BALANCE</Label>
    {@render matchOption(uOnEvmBalance, renderBalance, renderBalanceSkeleton)}
  </div>

  <div>
    <Input
      id="bondInput"
      type="text"
      required
      disabled={O.isNone(uOnEvmBalance)}
      label="Bond Amount"
      autocorrect="off"
      placeholder="Enter amount"
      spellcheck="false"
      autocomplete="off"
      inputmode="decimal"
      data-field="amount"
      onbeforeinput={(event) => {
        const { inputType, data, currentTarget } = event
        const { value } = currentTarget
        const proposed = value + (data ?? "")

        const maxDecimals = pipe(
          uOnEvmToken,
          O.map(Struct.get("representations")),
          O.flatMap(A.head),
          O.map(Struct.get("decimals")),
          O.getOrElse(() => 18),
        )

        const validShape = /^\d*[.,]?\d*$/.test(proposed)
        const validDecimalsDot = !proposed.includes(".")
          || proposed.split(".")[1].length <= maxDecimals
        const validDecimalsComma = !proposed.includes(",")
          || proposed.split(",")[1].length <= maxDecimals
        const isDelete = inputType.startsWith("delete")
        const validDecimals = validDecimalsComma && validDecimalsDot
        const noDuplicateLeadingZeroes = !proposed.startsWith("00")

        const allow = isDelete
          || (validDecimals && validShape && noDuplicateLeadingZeroes)

        if (!allow) {
          event.preventDefault()
        }
      }}
      autocapitalize="none"
      pattern="^[0-9]*[.,]?[0-9]*$"
      value={bondInput}
      class="h-14 text-center text-lg"
      oninput={(event) => {
        bondInput = event.currentTarget.value
      }}
    />
    {bondAmount}
  </div>

  <!-- Real-time Staking Rates -->
  <div class="flex flex-col gap-2 text-xs">
    {#if O.isSome(stakingRates.current) && stakingRates.current.value._tag === "Success"}
      <div class="flex justify-between">
        <span class="text-zinc-400">Purchase rate:</span>
        <span class="font-mono text-zinc-300">
          {
            pipe(
              stakingRates.current.value.value.purchase_rate,
              Utils.formatBigDecimal,
            )
          }
        </span>
      </div>
      <div class="flex justify-between items-center">
        <span class="text-zinc-400">Mint amount:</span>
        <div class="flex items-center gap-1">
          <button
            class={cn(
              "px-2 py-1 text-xs border rounded transition-colors",
              slippage === 1
                ? "border-blue-500 bg-blue-500/10 text-blue-400"
                : "border-zinc-700 bg-zinc-900 text-zinc-400 hover:border-zinc-600",
            )}
            onclick={() => slippage = 1}
          >
            1%
          </button>
          <button
            class={cn(
              "px-2 py-1 text-xs border rounded transition-colors",
              slippage === 2
                ? "border-blue-500 bg-blue-500/10 text-blue-400"
                : "border-zinc-700 bg-zinc-900 text-zinc-400 hover:border-zinc-600",
            )}
            onclick={() => slippage = 2}
          >
            2%
          </button>
          <button
            class={cn(
              "px-2 py-1 text-xs border rounded transition-colors",
              slippage === 3
                ? "border-blue-500 bg-blue-500/10 text-blue-400"
                : "border-zinc-700 bg-zinc-900 text-zinc-400 hover:border-zinc-600",
            )}
            onclick={() => slippage = 3}
          >
            3%
          </button>
        </div>
      </div>
      <div class="flex justify-between">
        <span class="text-zinc-400">Min you'll receive:</span>
        <span class="font-mono text-zinc-300">
          {
            pipe(
              minimumReceiveAmount,
              O.map(bd => Utils.formatBigDecimal(bd)),
              O.getOrElse(() => "0"),
            )
          } eU
        </span>
      </div>
    {:else}
      <div class="flex justify-between">
        <span class="text-zinc-400">Loading rates...</span>
        <div class="w-16 h-4 bg-zinc-700/50 rounded animate-pulse"></div>
      </div>
    {/if}
  </div>

  <!-- Status Display -->
  {#if !BondState.$is("Ready")(bondState)}
    <div class="bg-zinc-950/50 rounded-lg p-4 border border-zinc-800">
      <div class="flex items-center gap-3">
        <div class="size-8 rounded-lg {isError ? 'bg-red-500/20 border-red-500/40' : isSuccess ? 'bg-emerald-500/20 border-emerald-500/40' : 'bg-blue-500/20 border-blue-500/40'} flex items-center justify-center flex-shrink-0">
          {#if isBonding}
            <div class="w-4 h-4 border-2 border-blue-400 border-t-transparent rounded-full animate-spin">
            </div>
          {:else if isSuccess}
            <svg
              class="w-4 h-4 text-emerald-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M5 13l4 4L19 7"
              />
            </svg>
          {:else if isError}
            <svg
              class="w-4 h-4 text-red-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 9v2m0 4h.01"
              />
            </svg>
          {/if}
        </div>
        <div class="flex-1">
          <div class="text-sm font-medium text-white">
            {
              Match.value(bondState).pipe(
                Match.when(BondState.$is("SwitchingChain"), () => {
                  const isSafeWallet = getLastConnectedWalletId() === "safe"
                  return isSafeWallet
                    ? "Preparing Safe Transaction"
                    : "Switching to Sepolia"
                }),
                Match.when(BondState.$is("CheckingAllowance"), () =>
                  "Checking Token Allowance"),
                Match.when(BondState.$is("ApprovingAllowance"), () =>
                  "Approving Token Spending"),
                Match.when(BondState.$is("AllowanceSubmitted"), () =>
                  "Allowance Submitted"),
                Match.when(BondState.$is("WaitingForAllowanceConfirmation"), () =>
                  "Allowance Confirming"),
                Match.when(BondState.$is("AllowanceApproved"), () =>
                  "Allowance Approved"),
                Match.when(BondState.$is("CreatingTokenOrder"), () =>
                  "Creating Token Order"),
                Match.when(BondState.$is("PreparingBondTransaction"), () =>
                  "Preparing Bond Transaction"),
                Match.when(BondState.$is("ConfirmingBond"), () =>
                  "Confirming Bond"),
                Match.when(BondState.$is("BondSubmitted"), () =>
                  "Bond Submitted"),
                Match.when(BondState.$is("WaitingForConfirmation"), () =>
                  "Transaction Confirming"),
                Match.when(BondState.$is("WaitingForIndexer"), () =>
                  "Indexing Transaction"),
                Match.when(BondState.$is("Success"), () =>
                  "Bond Successful"),
                Match.when(BondState.$is("Error"), () =>
                  "Bond Failed"),
                Match.when(BondState.$is("Ready"), () =>
                  "Ready"),
                Match.exhaustive,
              )
            }
          </div>
          <div class="text-xs {isError ? 'text-red-400' : isSuccess ? 'text-emerald-400' : 'text-blue-400'} mt-1">
            {
              Match.value(bondState).pipe(
                Match.when(BondState.$is("SwitchingChain"), () => {
                  const isSafeWallet = getLastConnectedWalletId() === "safe"
                  return isSafeWallet
                    ? "Preparing transaction for Safe wallet..."
                    : "Please switch to Sepolia network in your wallet"
                }),
                Match.when(BondState.$is("CheckingAllowance"), () =>
                  "Reading current token allowance from blockchain..."),
                Match.when(BondState.$is("ApprovingAllowance"), () =>
                  "Confirm token approval transaction in your wallet"),
                Match.when(BondState.$is("AllowanceSubmitted"), ({ txHash }) =>
                  `Allowance transaction submitted: ${txHash.slice(0, 10)}...`),
                Match.when(
                  BondState.$is("WaitingForAllowanceConfirmation"),
                  ({ txHash }) =>
                    `Waiting for allowance confirmation: ${txHash.slice(0, 10)}...`,
                ),
                Match.when(BondState.$is("AllowanceApproved"), () =>
                  "Token spending approved, proceeding..."),
                Match.when(BondState.$is("CreatingTokenOrder"), () =>
                  "Building cross-chain token order..."),
                Match.when(BondState.$is("PreparingBondTransaction"), () =>
                  "Preparing bond transaction with contracts..."),
                Match.when(BondState.$is("ConfirmingBond"), () =>
                  "Confirm bond transaction in your wallet"),
                Match.when(BondState.$is("BondSubmitted"), ({ txHash }) =>
                  `Transaction submitted: ${txHash.slice(0, 10)}...`),
                Match.when(BondState.$is("WaitingForConfirmation"), ({ txHash }) =>
                  `Waiting for confirmation: ${txHash.slice(0, 10)}...`),
                Match.when(BondState.$is("WaitingForIndexer"), ({ txHash }) =>
                  `Transaction confirmed, indexing data...`),
                Match.when(BondState.$is("Success"), ({ txHash }) =>
                  `Success! TX: ${txHash.slice(0, 10)}...`),
                Match.when(BondState.$is("Error"), ({ message }) =>
                  message),
                Match.when(BondState.$is("Ready"), () =>
                  ""),
                Match.exhaustive,
              )
            }
          </div>
        </div>
      </div>
    </div>
  {/if}

  <div>
    <Button
      class="w-full"
      variant={isError ? "secondary" : "primary"}
      disabled={isBonding || isSuccess || O.isNone(bondAmount) || O.isNone(WalletStore.evmAddress)}
      onclick={isError ? handleRetry : handleBondSubmit}
    >
      {#if isBonding}
        <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin mr-2">
        </div>
      {:else if isSuccess}
        <svg
          class="w-4 h-4 text-current mr-2"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M5 13l4 4L19 7"
          />
        </svg>
      {/if}
      {
        Match.value(bondState).pipe(
          Match.when(BondState.$is("Ready"), () =>
            O.isNone(WalletStore.evmAddress)
              ? "Connect Wallet"
              : `Stake ${bondInput || "0"} U`),
          Match.when(BondState.$is("SwitchingChain"), () => "Switching..."),
          Match.when(BondState.$is("CheckingAllowance"), () => "Checking..."),
          Match.when(BondState.$is("ApprovingAllowance"), () => "Confirm in Wallet"),
          Match.when(BondState.$is("AllowanceSubmitted"), () => "Submitted"),
          Match.when(
            BondState.$is("WaitingForAllowanceConfirmation"),
            () => "Confirming...",
          ),
          Match.when(BondState.$is("AllowanceApproved"), () => "Approved ✓"),
          Match.when(BondState.$is("CreatingTokenOrder"), () => "Creating Order..."),
          Match.when(BondState.$is("PreparingBondTransaction"), () => "Preparing..."),
          Match.when(BondState.$is("ConfirmingBond"), () => "Confirm in Wallet"),
          Match.when(BondState.$is("BondSubmitted"), () => "Submitted"),
          Match.when(BondState.$is("WaitingForConfirmation"), () => "Confirming..."),
          Match.when(BondState.$is("WaitingForIndexer"), () => "Indexing..."),
          Match.when(BondState.$is("Success"), () => "Success!"),
          Match.when(BondState.$is("Error"), () => "Try Again"),
          Match.exhaustive,
        )
      }
    </Button>
  </div>
</div>
