<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Label from "$lib/components/ui/Label.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import UInput from "$lib/components/ui/UInput.svelte"
import { runPromiseExit$ } from "$lib/runtime"
import { getWagmiConnectorClient } from "$lib/services/evm/clients"
import { switchChain } from "$lib/services/transfer-ucs03-evm/chain"
import { uiStore } from "$lib/stores/ui.svelte"
import { wallets as WalletStore } from "$lib/stores/wallets.svelte"
import { safeOpts } from "$lib/transfer/shared/services/handlers/safe"
import { matchOption } from "$lib/utils/snippets.svelte"
import { getLastConnectedWalletId } from "$lib/wallet/evm/config.svelte"
import {
  Batch,
  Call,
  Indexer,
  Token,
  TokenOrder,
  Ucs05,
  Utils,
  ZkgmClient,
  ZkgmClientRequest,
  ZkgmIncomingMessage,
} from "@unionlabs/sdk"
import { Evm, EvmZkgmClient, Safe } from "@unionlabs/sdk-evm"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import {
  EU_ERC20,
  EU_LST,
  EU_SOLVER_ON_UNION_METADATA,
  EU_STAKING_HUB,
} from "@unionlabs/sdk/Constants"
import type { Chain, Token as TokenType } from "@unionlabs/sdk/schema"
import { TokenRawAmount, TokenRawDenom } from "@unionlabs/sdk/schema"
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
import { bytesToHex, custom, encodeAbiParameters, fromHex, http, keccak256 } from "viem"
import { mainnet } from "viem/chains"

const ETHEREUM_CHAIN_ID = UniversalChainId.make("ethereum.1")
const UNION_CHAIN_ID = UniversalChainId.make("union.union-1")
const SOURCE_CHANNEL_ID = ChannelId.make(2)
const DESTINATION_CHANNEL_ID = ChannelId.make(1)
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
  eUOnEvmToken: O.Option<TokenType>
  eUOnEvmBalance: O.Option<bigint>
  onUnbondSuccess?: () => void
}

let { evmChain, uOnEvmToken, eUOnEvmToken, eUOnEvmBalance, onUnbondSuccess }: Props = $props()

type UnbondState = Data.TaggedEnum<{
  Ready: {}
  SwitchingChain: {}
  CheckingAllowance: {}
  ApprovingAllowance: {}
  AllowanceSubmitted: { txHash: string }
  WaitingForAllowanceConfirmation: { txHash: string }
  AllowanceApproved: {}
  CreatingTokenOrder: {}
  PreparingUnbondTransaction: {}
  ConfirmingUnbond: {}
  UnbondSubmitted: { txHash: string }
  WaitingForConfirmation: { txHash: string }
  WaitingForIndexer: { txHash: string }
  Success: { txHash: string }
  Error: { message: string }
}>

const UnbondState = Data.taggedEnum<UnbondState>()

let unbondInput = $state<string>("")
let unbondAmount = $state<O.Option<bigint>>(O.none())
let unbondState = $state<UnbondState>(UnbondState.Ready())
let shouldUnbond = $state<boolean>(false)

const inputAmount = $derived<O.Option<BigDecimal.BigDecimal>>(pipe(
  unbondInput,
  BigDecimal.fromString,
))

const isUnbonding = $derived(
  !UnbondState.$is("Ready")(unbondState)
    && !UnbondState.$is("Success")(unbondState)
    && !UnbondState.$is("Error")(unbondState),
)
const isSuccess = $derived(UnbondState.$is("Success")(unbondState))
const isError = $derived(UnbondState.$is("Error")(unbondState))

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

    const encoded = yield* Effect.try(() => encodeAbiParameters(abi, args))

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
      0,
      ...u64toBeBytes(32n),
      ...fromHex(bytecode_base_checksum, "bytes"),
      ...u64toBeBytes(32n),
      ...fromHex(canonical_zkgm, "bytes"),
      ...u64toBeBytes(32n),
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

const checkAndSubmitAllowance = (sender: Ucs05.EvmDisplay, sendAmount: bigint) =>
  pipe(
    Evm.readErc20Allowance(
      EU_ERC20.address,
      sender.address,
      UCS03_EVM.address,
    ),
    Effect.tap(() =>
      Effect.sync(() => {
        unbondState = UnbondState.CheckingAllowance()
      })
    ),
    Effect.flatMap((amount) =>
      Effect.if(amount < sendAmount, {
        onTrue: () =>
          pipe(
            Effect.log(`Approving allowance ${sendAmount} for ${EU_ERC20.address}`),
            Effect.andThen(() =>
              Effect.sync(() => {
                unbondState = UnbondState.ApprovingAllowance()
              })
            ),
            Effect.andThen(() =>
              pipe(
                Evm.increaseErc20Allowance(
                  EU_ERC20.address,
                  UCS03_EVM,
                  sendAmount,
                ),
                Effect.tap((hash) =>
                  Effect.sync(() => {
                    unbondState = UnbondState.AllowanceSubmitted({ txHash: hash })
                  })
                ),
                Effect.tap(() => Effect.sleep("500 millis")),
                Effect.tap((hash) =>
                  Effect.sync(() => {
                    unbondState = UnbondState.WaitingForAllowanceConfirmation({
                      txHash: hash,
                    })
                  })
                ),
                Effect.andThen((hash) => Evm.waitForTransactionReceipt(hash as `0x${string}`)),
              )
            ),
          ),
        onFalse: () => Effect.log(`Allowance fulfilled for ${EU_ERC20.address}`),
      })
    ),
    Effect.tap(() =>
      Effect.sync(() => {
        unbondState = UnbondState.AllowanceApproved()
      })
    ),
    Effect.tap(() => Effect.sleep("500 millis")),
  )

const executeUnbond = (sender: Ucs05.EvmDisplay, sendAmount: bigint) =>
  Effect.gen(function*() {
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
    return yield* client.execute(request)
  })

runPromiseExit$(() =>
  shouldUnbond
    ? Effect.gen(function*() {
      const senderOpt = WalletStore.evmAddress
      if (O.isNone(senderOpt) || O.isNone(unbondAmount) || O.isNone(evmChain)) {
        unbondState = UnbondState.Error({
          message: "Missing required data: wallet address, unbond amount, or chain",
        })
        shouldUnbond = false
        return yield* Effect.fail(new Error("Missing required data"))
      }

      const sender = senderOpt.value
      const sendAmount = O.getOrThrow(unbondAmount)
      const chain = evmChain.value

      unbondState = UnbondState.SwitchingChain()

      const VIEM_CHAIN = mainnet

      const connectorClient = yield* getWagmiConnectorClient

      const isSafeWallet = getLastConnectedWalletId() === "safe"

      if (!isSafeWallet) {
        yield* switchChain(VIEM_CHAIN)
      }

      const maybeSafe = Match.value(isSafeWallet).pipe(
        Match.when(true, () => Safe.Safe.Default(safeOpts)),
        Match.when(false, () => Layer.empty),
        Match.exhaustive,
      )

      const publicClient = Evm.PublicClient.Live({
        chain: VIEM_CHAIN,
        transport: custom(connectorClient),
      })

      const walletClient = Evm.WalletClient.Live({
        account: connectorClient.account,
        chain: VIEM_CHAIN,
        transport: custom(connectorClient),
      })

      console.log("sender", sender)

      yield* checkAndSubmitAllowance(sender, sendAmount).pipe(
        Effect.provide(walletClient),
        Effect.provide(publicClient),
        Effect.provide(maybeSafe),
      )

      unbondState = UnbondState.ConfirmingUnbond()

      const executeBondWithProviders = executeUnbond(sender, sendAmount).pipe(
        Effect.provide(maybeSafe),
        Effect.provide(EvmZkgmClient.layerWithoutWallet),
        Effect.provide(walletClient),
        Effect.provide(publicClient),
        Effect.provide(ChainRegistry.Default),
      )

      const { txHash, safeHash } = yield* executeBondWithProviders

      unbondState = UnbondState.UnbondSubmitted({ txHash })
      yield* Effect.sleep("500 millis")

      unbondState = UnbondState.WaitingForConfirmation({ txHash })

      yield* pipe(
        Evm.waitForTransactionReceipt(txHash),
        Effect.provide(publicClient),
      )

      unbondState = UnbondState.WaitingForIndexer({ txHash })

      yield* pipe(
        Effect.gen(function*() {
          const indexer = yield* Indexer.Indexer
          return yield* indexer.fetch({
            document: graphql(`
              query GetUnbondByTxHash($tx_hash: String!) @cached(ttl: 10) {
                v2_unbonds(args: { p_transaction_hash: $tx_hash }) {
                  packet_hash
                }
              }
            `),
            variables: { tx_hash: txHash },
          })
        }),
        Effect.flatMap(Schema.decodeUnknown(
          Schema.Struct({
            v2_unbonds: Schema.NonEmptyArray(Schema.Struct({ packet_hash: Schema.String })),
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

      unbondState = UnbondState.Success({ txHash: txHash })

      unbondInput = ""
      shouldUnbond = false
      onUnbondSuccess?.()
      
      setTimeout(() => {
        if (UnbondState.$is("Success")(unbondState)) {
          unbondState = UnbondState.Ready()
        }
      }, 5000)

      return txHash
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

function handleButtonClick() {
  if (isUnbonding) return

  Match.value({ isError, isSuccess, hasWallet: O.isSome(WalletStore.evmAddress) }).pipe(
    Match.when({ isError: true }, () => {
      unbondState = UnbondState.Ready()
    }),
    Match.when({ isSuccess: true }, () => {
      unbondState = UnbondState.Ready()
    }),
    Match.when({ hasWallet: false }, () => {
      uiStore.openWalletModal()
    }),
    Match.orElse(() => {
      unbondState = UnbondState.Ready()
      shouldUnbond = true
    })
  )
}
</script>

{#snippet renderBalanceSkeleton()}
  <Skeleton class="w-full h-6 ml-auto" />
{/snippet}

{#snippet renderBalance(amount: bigint)}
  <div>
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
    <Label caseSensitive>BALANCE</Label>
    {#if O.isNone(WalletStore.evmAddress)}
      <div>—</div>
    {:else if O.isSome(evmChain) && O.isSome(eUOnEvmToken) && O.isSome(eUOnEvmBalance)}
      <TokenComponent
        chain={evmChain.value}
        denom={eUOnEvmToken.value.denom}
        amount={TokenRawAmount.make(eUOnEvmBalance.value)}
        showWrapping={false}
        showSymbol={true}
        showIcon={true}
      />
    {:else}
      {@render renderBalanceSkeleton()}
    {/if}
  </div>

  <div>
    <UInput
      id="unbondInput"
      label="Amount"
      placeholder="Enter amount"
      disabled={false}
      token={eUOnEvmToken}
      balance={eUOnEvmBalance}
      bind:humanValue={unbondInput}
      bind:weiValue={unbondAmount}
    />
  </div>

  <div class="flex justify-between items-center text-xs">
    <span class="text-zinc-400">Unbond time:</span>
    <span class="text-zinc-300">27 days</span>
  </div>

  <div class="flex justify-between items-center text-xs">
    <span class="text-zinc-400">You'll receive:</span>
    <div>
      {#if O.isSome(evmChain) && O.isSome(uOnEvmToken) && O.isSome(unbondAmount)}
        <TokenComponent
          chain={evmChain.value}
          denom={uOnEvmToken.value.denom}
          amount={TokenRawAmount.make(unbondAmount.value)}
          showWrapping={false}
          showSymbol={true}
          showIcon={true}
          maxDecimals={4}
        />
      {:else}
        <span class="text-zinc-300">0 U</span>
      {/if}
    </div>
  </div>

    <div class="bg-zinc-950/50 rounded-lg p-4 border border-zinc-800">
      <div class="flex items-center gap-3">
        <div class="size-8 rounded-lg {isError ? 'bg-red-500/20 border-red-500/40' : isSuccess ? 'bg-accent/20 border-accent/40' : UnbondState.$is("Ready")(unbondState) ? 'bg-zinc-700/20 border-zinc-600/40' : 'bg-accent/20 border-accent/40'} flex items-center justify-center flex-shrink-0">
          {#if UnbondState.$is("Ready")(unbondState)}
            <svg
              class="w-4 h-4 text-zinc-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          {:else if isUnbonding}
            <div class="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin">
            </div>
          {:else if isSuccess}
            <svg
              class="w-4 h-4 text-accent"
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
              Match.value(unbondState).pipe(
                Match.when(UnbondState.$is("Ready"), () =>
                  O.isNone(WalletStore.evmAddress)
                    ? "Connect your wallet to start unstaking"
                    : unbondInput
                    ? `Ready to unstake ${unbondInput} eU`
                    : "Enter amount to unstake eU tokens"),
                Match.when(UnbondState.$is("SwitchingChain"), () => {
                  const isSafeWallet = getLastConnectedWalletId() === "safe"
                  return isSafeWallet
                    ? "Preparing Safe Transaction"
                    : "Switching to mainnet"
                }),
                Match.when(
                  UnbondState.$is("CheckingAllowance"),
                  () => "Checking Token Allowance",
                ),
                Match.when(
                  UnbondState.$is("ApprovingAllowance"),
                  () => `Approving ${unbondInput || "0"} eU`,
                ),
                Match.when(
                  UnbondState.$is("AllowanceSubmitted"),
                  () => `Approval submitted`,
                ),
                Match.when(
                  UnbondState.$is("WaitingForAllowanceConfirmation"),
                  () => `Confirming submission`,
                ),
                Match.when(
                  UnbondState.$is("AllowanceApproved"),
                  () => `Approved ${unbondInput || "0"} eU`,
                ),
                Match.when(
                  UnbondState.$is("CreatingTokenOrder"),
                  () => `Creating order`,
                ),
                Match.when(
                  UnbondState.$is("PreparingUnbondTransaction"),
                  () => `Preparing unbond`,
                ),
                Match.when(
                  UnbondState.$is("ConfirmingUnbond"),
                  () => `Confirm unbond`,
                ),
                Match.when(UnbondState.$is("UnbondSubmitted"), () => `Unbond successfully submitted`),
                Match.when(
                  UnbondState.$is("WaitingForConfirmation"),
                  () => `Confirming submission`,
                ),
                Match.when(
                  UnbondState.$is("WaitingForIndexer"),
                  () => `Indexing submission`,
                ),
                Match.when(UnbondState.$is("Success"), () => `Unbond submitted`),
                Match.when(UnbondState.$is("Error"), () => "Unbond Failed"),
                Match.exhaustive,
              )
            }
          </div>
          <div class="text-xs {UnbondState.$is("Ready")(unbondState) ? 'text-zinc-400' : isError ? 'text-red-400' : isSuccess ? 'text-accent' : 'text-accent'} mt-1">
            {
              Match.value(unbondState).pipe(
                Match.when(UnbondState.$is("Ready"), () =>
                  O.isNone(WalletStore.evmAddress)
                    ? "Connect wallet to see balance and start unstaking"
                    : unbondInput
                    ? "Click unstake button to begin transaction (27 day unbond period)"
                    : "Enter the amount of eU tokens you want to unstake"),
                Match.when(UnbondState.$is("SwitchingChain"), () => {
                  const isSafeWallet = getLastConnectedWalletId() === "safe"
                  return isSafeWallet
                    ? "Preparing transaction for Safe wallet..."
                    : "Please switch to mainnet in your wallet"
                }),
                Match.when(UnbondState.$is("CheckingAllowance"), () =>
                  "Reading current token allowance from blockchain..."),
                Match.when(UnbondState.$is("ApprovingAllowance"), () =>
                  "Confirm token approval transaction in your wallet"),
                Match.when(UnbondState.$is("AllowanceSubmitted"), ({ txHash }) =>
                  `Allowance transaction submitted: ${txHash.slice(0, 10)}...`),
                Match.when(
                  UnbondState.$is("WaitingForAllowanceConfirmation"),
                  ({ txHash }) =>
                    `Waiting for allowance confirmation: ${txHash.slice(0, 10)}...`,
                ),
                Match.when(UnbondState.$is("AllowanceApproved"), () =>
                  "Token spending approved, proceeding..."),
                Match.when(UnbondState.$is("CreatingTokenOrder"), () =>
                  "Building cross-chain token order..."),
                Match.when(UnbondState.$is("PreparingUnbondTransaction"), () =>
                  "Preparing unbond transaction with contracts..."),
                Match.when(UnbondState.$is("ConfirmingUnbond"), () =>
                  "Confirm unbond transaction in your wallet"),
                Match.when(UnbondState.$is("UnbondSubmitted"), ({ txHash }) =>
                  `Transaction submitted: ${txHash.slice(0, 10)}...`),
                Match.when(UnbondState.$is("WaitingForConfirmation"), ({ txHash }) =>
                  `Waiting for confirmation: ${txHash.slice(0, 10)}...`),
                Match.when(UnbondState.$is("WaitingForIndexer"), ({ txHash }) =>
                  `Transaction confirmed, indexing data...`),
                Match.when(UnbondState.$is("Success"), ({ txHash }) =>
                  `Success! TX: ${txHash.slice(0, 10)}...`),
                Match.when(UnbondState.$is("Error"), ({ message }) =>
                  message),
                Match.exhaustive,
              )
            }
          </div>
        </div>
      </div>
    </div>

  <div>
    <Button
      class="w-full relative z-30"
      variant={isError ? "secondary" : "primary"}
      disabled={isUnbonding}
      onclick={handleButtonClick}
    >
      {#if isUnbonding}
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
        Match.value(unbondState).pipe(
          Match.when(UnbondState.$is("Ready"), () =>
            O.isNone(WalletStore.evmAddress)
              ? "Connect Wallet"
              : `Unstake ${unbondInput || "0"} eU`),
          Match.when(UnbondState.$is("SwitchingChain"), () => "Switching..."),
          Match.when(UnbondState.$is("CheckingAllowance"), () => "Checking..."),
          Match.when(UnbondState.$is("ApprovingAllowance"), () => "Confirm in Wallet"),
          Match.when(UnbondState.$is("AllowanceSubmitted"), () => "Submitted"),
          Match.when(UnbondState.$is("WaitingForAllowanceConfirmation"), () =>
            "Confirming..."),
          Match.when(UnbondState.$is("AllowanceApproved"), () =>
            "Approved ✓"),
          Match.when(UnbondState.$is("CreatingTokenOrder"), () => "Creating Order..."),
          Match.when(UnbondState.$is("PreparingUnbondTransaction"), () => "Preparing..."),
          Match.when(UnbondState.$is("ConfirmingUnbond"), () => "Confirm in Wallet"),
          Match.when(UnbondState.$is("UnbondSubmitted"), () => "Submitted"),
          Match.when(UnbondState.$is("WaitingForConfirmation"), () => "Confirming..."),
          Match.when(UnbondState.$is("WaitingForIndexer"), () => "Indexing..."),
          Match.when(UnbondState.$is("Success"), () => "Unstake Again"),
          Match.when(UnbondState.$is("Error"), () => "Try Again"),
          Match.exhaustive,
        )
      }
    </Button>
  </div>
</div>
