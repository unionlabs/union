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
  UCS03_MINTER_ON_UNION,
  UCS03_ZKGM,
  UNION_CHAIN_ID,
} from "$lib/stake/config"
import { predictProxy } from "$lib/stake/instantiate2"
import { type StakingRates } from "$lib/stake/schemas"
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
  Ucs03,
  Ucs05,
  Utils,
  ZkgmClient,
  ZkgmClientRequest,
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
import { TokenRawAmount } from "@unionlabs/sdk/schema"
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
import SlippageSelector from "./SlippageSelector.svelte"
import StatusDisplay from "./StatusDisplay.svelte"

const UCS03_EVM = Ucs05.EvmDisplay.make({ address: UCS03_EVM_ADDRESS })

interface Props {
  evmChain: O.Option<Chain>
  uOnEvmToken: O.Option<TokenType>
  eUOnEvmToken: O.Option<TokenType>
  uOnEvmBalance: O.Option<bigint>
  bondAmount: O.Option<bigint>
  stakingRates: O.Option<StakingRates>
  onBondSuccess?: () => void
}

let {
  evmChain,
  uOnEvmToken,
  eUOnEvmToken,
  uOnEvmBalance,
  bondAmount = $bindable(),
  stakingRates,
  onBondSuccess,
}: Props = $props()

const JsonFromBase64 = Schema.compose(
  Schema.StringFromBase64,
  Schema.parseJson(),
)

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
let slippage = $state<number>(0.5)

const isBonding = $derived(
  !BondState.$is("Ready")(bondState)
    && !BondState.$is("Success")(bondState)
    && !BondState.$is("Error")(bondState),
)
const isSuccess = $derived(BondState.$is("Success")(bondState))
const isError = $derived(BondState.$is("Error")(bondState))

// Derived state for button disabled logic
const isButtonDisabled = $derived(
  pipe(
    O.all([WalletStore.evmAddress, bondAmount, uOnEvmBalance]),
    O.match({
      onNone: () => isBonding, // If wallet not connected or data not loaded, only disable if bonding
      onSome: ([_, amount, balance]) => {
        // When wallet is connected and we have data
        return isBonding
          || amount === 0n
          || amount >= balance
      },
    }),
  ),
)

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

// Minimum receive amount as BigDecimal (with rate and slippage applied)
const minimumReceiveAmount = $derived<O.Option<BigDecimal.BigDecimal>>(pipe(
  O.Do,
  O.bind("input", () => inputAmount),
  O.bind("rates", () => stakingRates),
  O.map(({ input, rates }) => {
    const inputNorm = BigDecimal.normalize(input)
    const rateNorm = BigDecimal.normalize(rates.purchase_rate)

    const expectedScaled = inputNorm.value * rateNorm.value
    // Apply slippage
    const slippageBasisPoints = Math.floor(slippage * 100)
    const minScaled = expectedScaled * BigInt(10000 - slippageBasisPoints) / 10000n

    return BigDecimal.make(minScaled, inputNorm.scale + rateNorm.scale)
  }),
))

const checkAndSubmitAllowance = (sender: Ucs05.EvmDisplay, sendAmount: bigint) =>
  pipe(
    Evm.readErc20Allowance(
      U_ERC20.address,
      sender.address,
      UCS03_EVM.address,
    ),
    Effect.tap(() =>
      Effect.sync(() => {
        bondState = BondState.CheckingAllowance()
      })
    ),
    Effect.flatMap((amount) =>
      Effect.if(amount < sendAmount, {
        onTrue: () =>
          pipe(
            Effect.log(`Approving allowance ${sendAmount} for ${U_ERC20.address}`),
            Effect.andThen(() =>
              Effect.sync(() => {
                bondState = BondState.ApprovingAllowance()
              })
            ),
            Effect.andThen(() =>
              pipe(
                Evm.increaseErc20Allowance(
                  U_ERC20.address,
                  UCS03_EVM,
                  sendAmount,
                ),
                Effect.tap((hash) =>
                  Effect.sync(() => {
                    bondState = BondState.AllowanceSubmitted({ txHash: hash })
                  })
                ),
                Effect.tap(() => Effect.sleep("500 millis")),
                Effect.tap((hash) =>
                  Effect.sync(() => {
                    bondState = BondState.WaitingForAllowanceConfirmation({ txHash: hash })
                  })
                ),
                Effect.andThen((hash) => Evm.waitForTransactionReceipt(hash as `0x${string}`)),
              )
            ),
          ),
        onFalse: () => Effect.log(`Allowance fulfilled for ${U_ERC20.address}`),
      })
    ),
    Effect.tap(() =>
      Effect.sync(() => {
        bondState = BondState.AllowanceApproved()
      })
    ),
    Effect.tap(() => Effect.sleep("500 millis")),
  )

const executeBond = (sender: Ucs05.EvmDisplay, sendAmount: bigint, minMintAmount: bigint) =>
  Effect.gen(function*() {
    // minMintAmount is already calculated with purchase rate and slippage applied

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
          mint_to_address: proxy.address,
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
        sender: proxy,
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
  shouldBond
    ? Effect.gen(function*() {
      const validatedData = O.all({
        sender: WalletStore.evmAddress,
        sendAmount: bondAmount,
        chain: evmChain,
      })

      if (O.isNone(validatedData)) {
        bondState = BondState.Error({
          message: "Missing required data: wallet address, bond amount, or chain",
        })
        shouldBond = false
        return yield* Effect.fail(new Error("Missing required data"))
      }

      const { sender, sendAmount, chain } = validatedData.value

      bondState = BondState.SwitchingChain()

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

      yield* checkAndSubmitAllowance(sender, sendAmount).pipe(
        Effect.provide(walletClient),
        Effect.provide(publicClient),
        Effect.provide(maybeSafe),
      )

      bondState = BondState.CreatingTokenOrder()
      yield* Effect.sleep("300 millis")

      bondState = BondState.PreparingBondTransaction()
      yield* Effect.sleep("300 millis")

      bondState = BondState.ConfirmingBond()

      // Get the minimum receive amount and convert to wei for contract call
      if (O.isNone(minimumReceiveAmount)) {
        bondState = BondState.Error({ message: "Unable to calculate minimum receive amount" })
        return yield* Effect.fail(new Error("Unable to calculate minimum receive amount"))
      }

      const minMintAmountWei = Utils.toRawAmount(minimumReceiveAmount.value)

      const executeBondWithProviders = executeBond(sender, sendAmount, minMintAmountWei).pipe(
        Effect.provide(EvmZkgmClient.layerWithoutWallet),
        Effect.provide(walletClient),
        Effect.provide(publicClient),
        Effect.provide(ChainRegistry.Default),
        Effect.provide(maybeSafe),
      )

      const { txHash } = yield* executeBondWithProviders

      bondState = BondState.BondSubmitted({ txHash })
      yield* Effect.sleep("500 millis")

      bondState = BondState.WaitingForConfirmation({ txHash })

      yield* pipe(
        Evm.waitForTransactionReceipt(txHash),
        Effect.provide(publicClient),
      )

      bondState = BondState.WaitingForIndexer({ txHash })

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
            variables: { tx_hash: txHash },
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

      bondState = BondState.Success({ txHash: txHash })

      bondInput = ""
      shouldBond = false
      onBondSuccess?.()

      setTimeout(() => {
        if (BondState.$is("Success")(bondState)) {
          bondState = BondState.Ready()
        }
      }, 5000)
    }).pipe(
      Effect.catchAll(error =>
        Effect.gen(function*() {
          const errorDetails = extractErrorDetails(error) as any
          const fullError = errorDetails?.cause?.cause?.shortMessage
            || errorDetails?.cause?.message
            || errorDetails?.message
            || JSON.stringify(errorDetails)
          const shortMessage = String(fullError).split(".")[0]

          bondState = BondState.Error({ message: shortMessage })
          shouldBond = false
          return yield* Effect.void
        })
      ),
    )
    : Effect.void
)

function handleButtonClick() {
  if (isBonding) {
    return
  }

  Match.value({ isError, isSuccess, hasWallet: O.isSome(WalletStore.evmAddress) }).pipe(
    Match.when({ isError: true }, () => {
      bondState = BondState.Ready()
    }),
    Match.when({ isSuccess: true }, () => {
      bondState = BondState.Ready()
    }),
    Match.when({ hasWallet: false }, () => {
      uiStore.openWalletModal()
    }),
    Match.orElse(() => {
      bondState = BondState.Ready()
      shouldBond = true
    }),
  )
}
</script>

<div class="flex grow flex-col gap-4">
  <!-- Input Section with Balance -->
  <div class="space-y-3">
    <div class="flex justify-between items-center">
      <label
        for="bondInput"
        class="text-xs font-medium text-zinc-400 uppercase tracking-wider"
      >Amount to Stake</label>
      <div class="text-xs text-zinc-500 flex items-center gap-1">
        <span>Balance:</span>
        {#if O.isNone(WalletStore.evmAddress)}
          <span class="text-zinc-400">—</span>
        {:else if O.isSome(evmChain) && O.isSome(uOnEvmToken) && O.isSome(uOnEvmBalance)}
          <TokenComponent
            chain={evmChain.value}
            denom={uOnEvmToken.value.denom}
            amount={TokenRawAmount.make(uOnEvmBalance.value)}
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
        id="bondInput"
        label=""
        placeholder="0.0"
        disabled={O.isNone(uOnEvmBalance)}
        token={uOnEvmToken}
        balance={uOnEvmBalance}
        bind:humanValue={bondInput}
        bind:weiValue={bondAmount}
      />

      <!-- Quick Percentage Buttons -->
      <div class="mt-2">
        <QuickAmountButtons
          balance={uOnEvmBalance}
          decimals={18}
          onAmountSelect={(amount, wei) => {
            bondInput = amount
            bondAmount = O.some(wei)
          }}
        />
      </div>
    </div>
  </div>

  <!-- Transaction Preview Card -->
  <div class="rounded-lg bg-zinc-900 border border-zinc-800/50 p-3 space-y-3">
    {#if O.isSome(stakingRates)}
      <!-- Exchange Rate -->
      <div class="flex justify-between items-center">
        <span class="text-xs text-zinc-500">Exchange Rate</span>
        <span class="text-sm font-medium text-zinc-200">
          1 U = {
            pipe(
              stakingRates.value.purchase_rate,
              BigDecimal.round({ mode: "from-zero", scale: 6 }),
              Utils.formatBigDecimal,
            )
          } eU
        </span>
      </div>

      <!-- Slippage Settings -->
      <SlippageSelector
        value={slippage}
        onchange={(value) => slippage = value}
      />

      <!-- You'll Receive -->
      <div class="pt-2 border-t border-zinc-800">
        <div class="flex justify-between items-center">
          <span class="text-xs text-zinc-500">You'll Receive</span>
          <div class="text-right">
            {#if O.isSome(evmChain) && O.isSome(eUOnEvmToken)
              && O.isSome(minimumReceiveAmount)}
              <TokenComponent
                chain={evmChain.value}
                denom={eUOnEvmToken.value.denom}
                amount={TokenRawAmount.make(Utils.toRawAmount(minimumReceiveAmount.value))}
                showWrapping={false}
                showSymbol={true}
                showIcon={true}
                maxDecimals={4}
              />
            {:else}
              <span class="text-zinc-300">
                {
                  pipe(
                    minimumReceiveAmount,
                    O.map(bd => Utils.formatBigDecimal(bd)),
                    O.getOrElse(() => "0"),
                  )
                } eU
              </span>
            {/if}
          </div>
        </div>
      </div>
    {:else}
      <!-- Loading state for Exchange Rate -->
      <div class="flex justify-between items-center">
        <span class="text-xs text-zinc-500">Exchange Rate</span>
        <span class="text-sm font-medium">
          <Skeleton class="inline-block w-24 h-5" />
        </span>
      </div>

      <!-- Loading state for Slippage Settings -->
      <div class="space-y-2">
        <div class="flex justify-between items-center">
          <span class="text-xs text-zinc-500">Slippage</span>
          <div class="flex gap-1">
            <Skeleton class="w-12 h-7" />
            <Skeleton class="w-12 h-7" />
            <Skeleton class="w-12 h-7" />
          </div>
        </div>
      </div>

      <!-- Loading state for You'll Receive -->
      <div class="pt-2 border-t border-zinc-800">
        <div class="flex justify-between items-center">
          <span class="text-xs text-zinc-500">You'll Receive</span>
          <div class="text-right">
            <div class="flex items-center gap-1.5 justify-end">
              <Skeleton class="w-5 h-5 rounded-full" />
              <Skeleton class="w-20 h-5" />
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>

  <!-- Status Display -->
  <StatusDisplay
    state={bondState}
    type="bond"
    inputAmount={bondInput}
  />

  <!-- Action Button -->
  <Button
    variant={isError ? "secondary" : "primary"}
    disabled={isButtonDisabled}
    onclick={handleButtonClick}
  >
    {#if isBonding}
      <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin mr-2">
      </div>
    {/if}
    {
      Match.value(bondState).pipe(
        Match.when(BondState.$is("Ready"), () =>
          O.isNone(WalletStore.evmAddress)
            ? "Connect Wallet"
            : bondInput
            ? "Stake"
            : "Enter Amount"),
        Match.when(BondState.$is("SwitchingChain"), () => "Switching..."),
        Match.when(BondState.$is("CheckingAllowance"), () => "Checking..."),
        Match.when(BondState.$is("ApprovingAllowance"), () => "Approve in Wallet"),
        Match.when(BondState.$is("AllowanceSubmitted"), () => "Processing..."),
        Match.when(BondState.$is("WaitingForAllowanceConfirmation"), () => "Confirming..."),
        Match.when(BondState.$is("AllowanceApproved"), () => "Approved ✓"),
        Match.when(BondState.$is("CreatingTokenOrder"), () => "Creating..."),
        Match.when(BondState.$is("PreparingBondTransaction"), () => "Preparing..."),
        Match.when(BondState.$is("ConfirmingBond"), () => "Confirm in Wallet"),
        Match.when(BondState.$is("BondSubmitted"), () => "Processing..."),
        Match.when(BondState.$is("WaitingForConfirmation"), () => "Confirming..."),
        Match.when(BondState.$is("WaitingForIndexer"), () => "Finalizing..."),
        Match.when(BondState.$is("Success"), () => "Stake Again"),
        Match.when(BondState.$is("Error"), () => "Try Again"),
        Match.exhaustive,
      )
    }
  </Button>
</div>
