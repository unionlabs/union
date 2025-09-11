<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Input from "$lib/components/ui/Input.svelte"
import Label from "$lib/components/ui/Label.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { runPromiseExit$ } from "$lib/runtime"
import { getWagmiConnectorClient } from "$lib/services/evm/clients"
import { switchChain } from "$lib/services/transfer-ucs03-evm/chain"
import { wallets as WalletStore } from "$lib/stores/wallets.svelte"
import { matchOption } from "$lib/utils/snippets.svelte"
import {
  Batch,
  Call,
  Token,
  TokenOrder,
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
  EU_LST,
  EU_SOLVER_ON_UNION_METADATA,
  EU_STAKING_HUB,
} from "@unionlabs/sdk/Constants"
import type { Chain, Token as TokenType } from "@unionlabs/sdk/schema"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import { HexFromJson } from "@unionlabs/sdk/schema/hex"
import { BigDecimal, Data, Effect, Exit, Schedule, Match, pipe, Schema, Struct } from "effect"
import * as A from "effect/Array"
import { flow } from "effect/Function"
import * as O from "effect/Option"
import { bytesToHex, encodeAbiParameters, fromHex, keccak256, custom, http } from "viem"
import { holesky } from "viem/chains"

// Constants from unbond.ts
const ETHEREUM_CHAIN_ID = UniversalChainId.make("ethereum.17000")
const UNION_CHAIN_ID = UniversalChainId.make("union.union-testnet-10")
const SOURCE_CHANNEL_ID = ChannelId.make(6)
const DESTINATION_CHANNEL_ID = ChannelId.make(20)
const UCS03_EVM = Ucs05.EvmDisplay.make({
  address: "0x5fbe74a283f7954f10aa04c2edf55578811aeb03",
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
  eUOnEvmBalance: O.Option<bigint>
  onUnbondSuccess?: () => void
}

let { evmChain, uOnEvmToken, eUOnEvmBalance, onUnbondSuccess }: Props = $props()

type UnbondState = Data.TaggedEnum<{
  Ready: {}
  SwitchingChain: {}
  CheckingAllowance: {}
  ApprovingAllowance: {}
  AllowanceApproved: {}
  CreatingTokenOrder: {}
  PreparingUnbondTransaction: {}
  ExecutingUnbond: {}
  WaitingForTxConfirmation: {}
  WaitingForIndexer: {}
  Success: { txHash: string }
  Error: { message: string }
}>

const UnbondState = Data.taggedEnum<UnbondState>()

let unbondInput = $state<string>("")
let unbondState = $state<UnbondState>(UnbondState.Ready())
let shouldUnbond = $state<boolean>(false)

// Human readable amount as BigDecimal (maintains precision)
const inputAmount = $derived<O.Option<BigDecimal.BigDecimal>>(pipe(
  unbondInput,
  BigDecimal.fromString,
))

// Wei amount for blockchain - properly convert BigDecimal to wei
const unbondAmount = $derived<O.Option<bigint>>(pipe(
  inputAmount,
  O.map(flow(
    (bd) => {
      const multiplier = BigDecimal.make(10n ** 18n, 0)
      const result = BigDecimal.multiply(bd, multiplier)
      console.log("Unbond conversion - Input:", unbondInput, "Result value:", result.value)
      return result.value // Don't normalize - we want the raw value
    }
  )),
))

const isUnbonding = $derived(
  UnbondState.$is("SwitchingChain")(unbondState) ||
  UnbondState.$is("CheckingAllowance")(unbondState) ||
  UnbondState.$is("ApprovingAllowance")(unbondState) ||
  UnbondState.$is("AllowanceApproved")(unbondState) ||
  UnbondState.$is("CreatingTokenOrder")(unbondState) ||
  UnbondState.$is("PreparingUnbondTransaction")(unbondState) ||
  UnbondState.$is("ExecutingUnbond")(unbondState) ||
  UnbondState.$is("WaitingForTxConfirmation")(unbondState) ||
  UnbondState.$is("WaitingForIndexer")(unbondState)
)
const isSuccess = $derived(UnbondState.$is("Success")(unbondState))
const isError = $derived(UnbondState.$is("Error")(unbondState))

// Helper functions from unbond.ts
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

    const encoded = yield* Effect.try(() =>
      encodeAbiParameters(abi, args)
    )

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
    const r2 = yield* Schema.decode(Ucs05.Bech32FromCanonicalBytesWithPrefix("union"))(rBytes)

    return Ucs05.CosmosDisplay.make({ address: r2 })
  },
)

const checkAndSubmitAllowance = (sender: Ucs05.EvmDisplay, sendAmount: bigint) => pipe(
  Evm.readErc20Allowance(
    EU_ERC20.address,
    sender.address,
    UCS03_EVM.address,
  ),
  Effect.flatMap((amount) =>
    Effect.if(amount < sendAmount, {
      onTrue: () =>
        pipe(
          Effect.log(`Increasing allowance by ${sendAmount - amount} for ${EU_ERC20.address}`),
          Effect.andThen(() =>
            pipe(
              Evm.increaseErc20Allowance(
                EU_ERC20.address,
                UCS03_EVM,
                sendAmount - amount,
              ),
              Effect.andThen(Evm.waitForTransactionReceipt),
            )
          ),
        ),
      onFalse: () =>
        Effect.log(`Allowance fulfilled for ${EU_ERC20.address}`),
    })
  ),
)

const executeUnbond = (sender: Ucs05.EvmDisplay, sendAmount: bigint) => Effect.gen(function*() {
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
    baseToken: EU_ERC20,
    baseAmount: sendAmount,
    quoteToken: Token.Cw20.make({ address: EU_LST.address }),
    quoteAmount: sendAmount,
    kind: "solve",
    metadata: EU_SOLVER_ON_UNION_METADATA,
    version: 2,
  })

  const increaseAllowanceCall = yield* pipe(
    {
      increase_allowance: {
        spender: EU_STAKING_HUB.address,
        amount: sendAmount,
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
  const response = yield* client.execute(request)
  
  yield* Effect.log("Submission TX Hash:", response.txHash)

  // Return both response and txHash for separate indexer handling
  return { response, txHash: response.txHash }
})

runPromiseExit$(() =>
  shouldUnbond
    ? Effect.gen(function*() {
      const senderOpt = WalletStore.evmAddress
      if (O.isNone(senderOpt) || O.isNone(unbondAmount) || O.isNone(evmChain)) {
        unbondState = UnbondState.Error({ message: "Missing required data: wallet address, unbond amount, or chain" })
        shouldUnbond = false
        return yield* Effect.fail(new Error("Missing required data"))
      }

      const sender = senderOpt.value
      const sendAmount = unbondAmount.value
      const chain = evmChain.value

      unbondState = UnbondState.SwitchingChain()
      yield* Effect.log("Starting unbond execution", { sender: sender.address, sendAmount })
      
      const RPC_URL = "https://rpc.17000.ethereum.chain.kitchen"
      const VIEM_CHAIN = holesky
      
      // Get wagmi connector client
      const connectorClient = yield* getWagmiConnectorClient
      
      // Switch to the correct chain
      yield* switchChain(VIEM_CHAIN)
      
      // Create clients using connector
      const publicClient = Evm.PublicClient.Live({
        chain: VIEM_CHAIN,
        transport: custom(connectorClient),
      })

      const walletClient = Evm.WalletClient.Live({
        account: connectorClient.account,
        chain: VIEM_CHAIN,
        transport: custom(connectorClient),
      })
      
      unbondState = UnbondState.CheckingAllowance()
      yield* checkAndSubmitAllowance(sender, sendAmount).pipe(
        Effect.provide(walletClient),
        Effect.provide(publicClient),
        Effect.tap(() => Effect.sync(() => { unbondState = UnbondState.ApprovingAllowance() }))
      )
      
      unbondState = UnbondState.ExecutingUnbond()
      const { response, txHash } = yield* executeUnbond(sender, sendAmount).pipe(
        Effect.provide(EvmZkgmClient.layerWithoutWallet),
        Effect.provide(walletClient),
        Effect.provide(publicClient),
        Effect.provide(ChainRegistry.Default),
      )
      
      // Unbond transaction completed, now wait for indexer
      console.log("Unbond transaction submitted with hash:", txHash)
      unbondState = UnbondState.WaitingForIndexer()
      
      // Wait for indexer with aggressive retry - the data WILL eventually be there
      const receipt = yield* Effect.retry(
        response.waitFor(
          ZkgmIncomingMessage.LifecycleEvent.$is("EvmTransactionReceiptComplete"),
        ),
        {
          schedule: pipe(Schedule.fixed("5 seconds"), Schedule.intersect(Schedule.recurs(30))),
          while: (error) => {
            console.log("Indexer not ready yet, retrying in 5 seconds...")
            return true // Always retry - indexer will eventually have the data
          }
        }
      )
      
      unbondState = UnbondState.Success({ txHash })
      
      // Reset form and refresh data on success
      unbondInput = ""
      shouldUnbond = false
      onUnbondSuccess?.()
      
      return receipt
    }).pipe(
      Effect.catchAll(error =>
        Effect.gen(function*() {
          const errorObj = error as any
          const fullError = errorObj?.cause?.cause?.shortMessage
            || errorObj?.cause?.message
            || errorObj?.message
            || JSON.stringify(error)
          const shortMessage = String(fullError).split(".")[0]
          
          unbondState = UnbondState.Error({ message: shortMessage })
          shouldUnbond = false
          return yield* Effect.succeed(false)
        })
      ),
    )
    : Effect.void
)

function handleUnbondSubmit() {
  if (isUnbonding) {
    return
  }
  unbondState = UnbondState.Ready()
  shouldUnbond = true
}

function handleRetry() {
  unbondState = UnbondState.Ready()
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
        // XXX: check decimals
        BigDecimal.unsafeDivide(BigDecimal.make(1n, -18)),
        Utils.formatBigDecimal,
      )
    }
  </div>
{/snippet}

<div class="flex grow flex-col gap-4">
  <div>
    <Label caseSensitive>eU BALANCE</Label>
    {@render matchOption(eUOnEvmBalance, renderBalance, renderBalanceSkeleton)}
  </div>

  <div>
    <Input
      id="unbondInput"
      type="text"
      required
      disabled={false}
      label="Unbond Amount"
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

        const a = pipe(
          S.BigDecimal,
          S.filter(
            (x) => x.scale <= 18,
            {
              description: "can have at most 18 decimals",
            },
          ),
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
      value={unbondInput}
      class="h-14 text-center text-lg"
      oninput={(event) => {
        unbondInput = event.currentTarget.value
      }}
    />
    {O.map(unbondAmount, (wei) => wei.toString())}
  </div>

  <!-- Status Display -->
  {#if !UnbondState.$is("Ready")(unbondState)}
    <div class="bg-zinc-950/50 rounded-lg p-4 border border-zinc-800">
      <div class="flex items-center gap-3">
        <div class="size-8 rounded-lg {isError ? 'bg-red-500/20 border-red-500/40' : isSuccess ? 'bg-emerald-500/20 border-emerald-500/40' : 'bg-blue-500/20 border-blue-500/40'} flex items-center justify-center flex-shrink-0">
          {#if isUnbonding}
            <div class="w-4 h-4 border-2 border-blue-400 border-t-transparent rounded-full animate-spin"></div>
          {:else if isSuccess}
            <svg class="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
            </svg>
          {:else if isError}
            <svg class="w-4 h-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01"/>
            </svg>
          {/if}
        </div>
        <div class="flex-1">
          <div class="text-sm font-medium text-white">
            {
              Match.value(unbondState).pipe(
                Match.when(UnbondState.$is("SwitchingChain"), () => "Switching to Holesky"),
                Match.when(UnbondState.$is("CheckingAllowance"), () => "Checking Token Allowance"),
                Match.when(UnbondState.$is("ApprovingAllowance"), () => "Approving Token Spending"),
                Match.when(UnbondState.$is("AllowanceApproved"), () => "Allowance Approved"),
                Match.when(UnbondState.$is("CreatingTokenOrder"), () => "Creating Token Order"),
                Match.when(UnbondState.$is("PreparingUnbondTransaction"), () => "Preparing Unbond Transaction"),
                Match.when(UnbondState.$is("ExecutingUnbond"), () => "Executing Unbond"),
                Match.when(UnbondState.$is("WaitingForTxConfirmation"), () => "Transaction Confirming"),
                Match.when(UnbondState.$is("WaitingForIndexer"), () => "Indexing Transaction"),
                Match.when(UnbondState.$is("Success"), () => "Unbond Successful"),
                Match.when(UnbondState.$is("Error"), () => "Unbond Failed"),
                Match.when(UnbondState.$is("Ready"), () => "Ready"),
                Match.exhaustive,
              )
            }
          </div>
          <div class="text-xs {isError ? 'text-red-400' : isSuccess ? 'text-emerald-400' : 'text-blue-400'} mt-1">
            {
              Match.value(unbondState).pipe(
                Match.when(UnbondState.$is("SwitchingChain"), () => "Please switch to Holesky network in your wallet"),
                Match.when(UnbondState.$is("CheckingAllowance"), () => "Reading current token allowance from blockchain..."),
                Match.when(UnbondState.$is("ApprovingAllowance"), () => "Confirm token approval transaction in your wallet"),
                Match.when(UnbondState.$is("AllowanceApproved"), () => "Token spending approved, proceeding..."),
                Match.when(UnbondState.$is("CreatingTokenOrder"), () => "Building cross-chain token order..."),
                Match.when(UnbondState.$is("PreparingUnbondTransaction"), () => "Preparing unbond transaction with contracts..."),
                Match.when(UnbondState.$is("ExecutingUnbond"), () => "Confirm unbond transaction in your wallet"),
                Match.when(UnbondState.$is("WaitingForTxConfirmation"), () => "Transaction submitted, waiting for confirmation..."),
                Match.when(UnbondState.$is("WaitingForIndexer"), () => "Transaction confirmed, indexing data..."),
                Match.when(UnbondState.$is("Success"), ({ txHash }) => `Success! TX: ${txHash.slice(0, 10)}...`),
                Match.when(UnbondState.$is("Error"), ({ message }) => message),
                Match.when(UnbondState.$is("Ready"), () => ""),
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
      disabled={isUnbonding || isSuccess || O.isNone(unbondAmount) || O.isNone(WalletStore.evmAddress)}
      onclick={isError ? handleRetry : handleUnbondSubmit}
    >
      {#if isUnbonding}
        <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin mr-2"></div>
      {:else if isSuccess}
        <svg class="w-4 h-4 text-current mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
        </svg>
      {/if}
      {
        Match.value(unbondState).pipe(
          Match.when(UnbondState.$is("Ready"), () => 
            O.isNone(WalletStore.evmAddress) 
              ? "Connect Wallet" 
              : `Unstake ${unbondInput || "0"} eU`
          ),
          Match.when(UnbondState.$is("SwitchingChain"), () => "Switching..."),
          Match.when(UnbondState.$is("CheckingAllowance"), () => "Checking..."),
          Match.when(UnbondState.$is("ApprovingAllowance"), () => "Approving..."),
          Match.when(UnbondState.$is("AllowanceApproved"), () => "Approved ✓"),
          Match.when(UnbondState.$is("CreatingTokenOrder"), () => "Creating Order..."),
          Match.when(UnbondState.$is("PreparingUnbondTransaction"), () => "Preparing..."),
          Match.when(UnbondState.$is("ExecutingUnbond"), () => "Executing..."),
          Match.when(UnbondState.$is("WaitingForTxConfirmation"), () => "Confirming..."),
          Match.when(UnbondState.$is("WaitingForIndexer"), () => "Indexing..."),
          Match.when(UnbondState.$is("Success"), () => "Success!"),
          Match.when(UnbondState.$is("Error"), () => "Try Again"),
          Match.exhaustive,
        )
      }
    </Button>
  </div>
</div>
