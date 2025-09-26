<script lang="ts">
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import Button from "$lib/components/ui/Button.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import UInput from "$lib/components/ui/UInput.svelte"
import { runPromiseExit$ } from "$lib/runtime"
import { getWagmiConnectorClient } from "$lib/services/evm/clients"
import { switchChain } from "$lib/services/transfer-ucs03-evm/chain"
import {
  DESTINATION_CHANNEL_ID,
  ETHEREUM_CHAIN_ID,
  SOURCE_CHANNEL_ID,
  UCS03_EVM_ADDRESS,
  UCS03_ZKGM,
  UNION_CHAIN_ID,
} from "$lib/stake/config"
import { predictProxy } from "$lib/stake/instantiate2"
import { type StakingRates, StakingRatesSchema } from "$lib/stake/schemas"
import { uiStore } from "$lib/stores/ui.svelte"
import { wallets as WalletStore } from "$lib/stores/wallets.svelte"
import { safeOpts } from "$lib/transfer/shared/services/handlers/safe"
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
import { HexFromJson } from "@unionlabs/sdk/schema/hex"
import { extractErrorDetails } from "@unionlabs/sdk/utils/index"
import {
  BigDecimal,
  ConfigProvider,
  Data,
  Effect,
  Exit,
  Layer,
  Match,
  pipe,
  Schedule,
  Schema,
} from "effect"
import * as O from "effect/Option"
import { graphql } from "gql.tada"
import { custom } from "viem"
import { mainnet } from "viem/chains"
import QuickAmountButtons from "./QuickAmountButtons.svelte"
import StatusDisplay from "./StatusDisplay.svelte"

const UCS03_EVM = Ucs05.EvmDisplay.make({ address: UCS03_EVM_ADDRESS })

interface Props {
  evmChain: O.Option<Chain>
  uOnEvmToken: O.Option<TokenType>
  eUOnEvmToken: O.Option<TokenType>
  eUOnEvmBalance: O.Option<bigint>
  stakingRates: O.Option<StakingRates>
  onUnbondSuccess?: () => void
}

let {
  evmChain,
  uOnEvmToken,
  eUOnEvmToken,
  eUOnEvmBalance,
  stakingRates,
  onUnbondSuccess,
}: Props = $props()

const JsonFromBase64 = Schema.compose(
  Schema.StringFromBase64,
  Schema.parseJson(),
)

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

const isUnbonding = $derived(
  !UnbondState.$is("Ready")(unbondState)
    && !UnbondState.$is("Success")(unbondState)
    && !UnbondState.$is("Error")(unbondState),
)
const isSuccess = $derived(UnbondState.$is("Success")(unbondState))
const isError = $derived(UnbondState.$is("Error")(unbondState))

// Derived state for button disabled logic
const isButtonDisabled = $derived(
  pipe(
    O.all([WalletStore.evmAddress, unbondAmount, eUOnEvmBalance]),
    O.match({
      onNone: () => isUnbonding, // If wallet not connected or data not loaded, only disable if unbonding
      onSome: ([_, amount, balance]) => {
        // When wallet is connected and we have data
        return isUnbonding
          || amount === 0n
          || amount > balance
      },
    }),
  ),
)

const QlpConfigProvider = pipe(
  ConfigProvider.fromMap(
    new Map([
      ["GRAPHQL_ENDPOINT", "https://graphql.union.build/v1/graphql"],
    ]),
  ),
  Layer.setConfigProvider,
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
    const proxy = yield* predictProxy({
      path: 0n,
      channel: DESTINATION_CHANNEL_ID,
      sender,
    })

    const tokenOrder = yield* TokenOrder.make({
      source: ethereumChain,
      destination: unionChain,
      sender,
      receiver: proxy,
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
          contractAddress: proxy,
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
      const validatedData = O.all({
        sender: WalletStore.evmAddress,
        sendAmount: unbondAmount,
        chain: evmChain,
      })

      if (O.isNone(validatedData)) {
        unbondState = UnbondState.Error({
          message: "Missing required data: wallet address, unbond amount, or chain",
        })
        shouldUnbond = false
        return yield* Effect.fail(new Error("Missing required data"))
      }

      const { sender, sendAmount, chain } = validatedData.value

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
        Effect.provide(EvmZkgmClient.layerWithoutWallet),
        Effect.provide(walletClient),
        Effect.provide(publicClient),
        Effect.provide(ChainRegistry.Default),
        Effect.provide(maybeSafe),
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
          const errorDetails = extractErrorDetails(error) as any
          const fullError = errorDetails?.cause?.cause?.shortMessage
            || errorDetails?.cause?.message
            || errorDetails?.message
            || JSON.stringify(errorDetails)
          const shortMessage = String(fullError).split(".")[0]

          unbondState = UnbondState.Error({ message: shortMessage })
          shouldUnbond = false
          return yield* Effect.void
        })
      ),
    )
    : Effect.void
)

function handleButtonClick() {
  if (isUnbonding) {
    return
  }

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
    }),
  )
}
</script>

<div class="flex grow flex-col gap-4">
  <!-- Input Section with Balance -->
  <div class="space-y-3">
    <div class="flex justify-between items-center">
      <label
        for="unbondInput"
        class="text-xs font-medium text-zinc-400 uppercase tracking-wider"
      >Amount to Unstake</label>
      <div class="text-xs text-zinc-500 flex items-center gap-1">
        <span>Balance:</span>
        {#if O.isNone(WalletStore.evmAddress)}
          <span class="text-zinc-400">—</span>
        {:else if O.isSome(evmChain) && O.isSome(eUOnEvmToken) && O.isSome(eUOnEvmBalance)}
          <TokenComponent
            chain={evmChain.value}
            denom={eUOnEvmToken.value.denom}
            amount={TokenRawAmount.make(eUOnEvmBalance.value)}
            showWrapping={false}
            showSymbol={true}
            showIcon={false}
          />
        {:else}
          <div class="flex items-center gap-1 font-semibold">
            <Skeleton class="w-20 h-4 inline-block" />
            <Skeleton class="w-6 h-4 inline-block" />
          </div>
        {/if}
      </div>
    </div>

    <div class="relative">
      <UInput
        id="unbondInput"
        label=""
        placeholder="0.0"
        disabled={false}
        token={eUOnEvmToken}
        balance={eUOnEvmBalance}
        bind:humanValue={unbondInput}
        bind:weiValue={unbondAmount}
      />

      <!-- Quick Percentage Buttons -->
      <div class="mt-2">
        <QuickAmountButtons
          balance={eUOnEvmBalance}
          decimals={18}
          onAmountSelect={(amount, wei) => {
            unbondInput = amount
            unbondAmount = O.some(wei)
          }}
        />
      </div>
    </div>
  </div>

  <div class="flex-1" />

  <!-- Transaction Preview Card -->
  <div class="rounded-lg bg-zinc-900 border border-zinc-800/50 p-3 space-y-3">
    <div class="flex justify-between items-center">
      <span class="text-xs text-zinc-500">Exchange Rate</span>
      {#if O.isSome(stakingRates)}
        <span class="text-sm font-medium text-zinc-200">
          1 eU = {
            pipe(
              stakingRates.value.redemption_rate,
              BigDecimal.round({ mode: "from-zero", scale: 6 }),
              Utils.formatBigDecimal,
            )
          } U
        </span>
      {:else}
        <span class="text-sm font-medium">
          <Skeleton class="inline-block w-24 h-5" />
        </span>
      {/if}
    </div>

    <div class="flex justify-between items-center">
      <span class="text-xs text-zinc-500">Unbond Period</span>
      <span class="text-sm font-medium text-zinc-200">27 days</span>
    </div>

    <div class="pt-2 border-t border-zinc-800">
      <div class="flex justify-between items-center">
        <span class="text-xs text-zinc-500">You'll Receive</span>
        <div class="text-right">
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
            <span class="text-sm font-medium text-zinc-300">0 U</span>
            <div class="text-xs text-zinc-500 mt-0.5">Enter amount</div>
          {/if}
        </div>
      </div>
    </div>
  </div>

  <!-- Status Display -->
  <StatusDisplay
    state={unbondState}
    type="unbond"
    inputAmount={unbondInput}
  />

  <div>
    <Button
      class="w-full relative z-30"
      variant={isError ? "secondary" : "primary"}
      disabled={isButtonDisabled}
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
              : unbondInput
              ? "Unstake"
              : "Enter Amount"),
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
