<script lang="ts">
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import Button from "$lib/components/ui/Button.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import UInput from "$lib/components/ui/UInput.svelte"
import { runPromiseExit$ } from "$lib/runtime"
import { getWagmiConnectorClient } from "$lib/services/evm/clients"
import { switchChain } from "$lib/services/transfer-ucs03-evm/chain"
import { uiStore } from "$lib/stores/ui.svelte"
import { wallets as WalletStore } from "$lib/stores/wallets.svelte"
import { safeOpts } from "$lib/transfer/shared/services/handlers/safe"
import { getLastConnectedWalletId } from "$lib/wallet/evm/config.svelte"
import { Utils, Ucs05 } from "@unionlabs/sdk"
import { Evm, Safe } from "@unionlabs/sdk-evm"
import { EU_ERC20, U_ERC20 } from "@unionlabs/sdk/Constants"
import type { Chain, Token as TokenType } from "@unionlabs/sdk/schema"
import { TokenRawAmount } from "@unionlabs/sdk/schema"
import { extractErrorDetails } from "@unionlabs/sdk/utils/index"
import {
  BigDecimal,
  Data,
  Effect,
  Layer,
  Match,
  pipe,
} from "effect"
import * as O from "effect/Option"
import { custom } from "viem"
import { mainnet } from "viem/chains"
import QuickAmountButtons from "./QuickAmountButtons.svelte"
import StatusDisplay from "./StatusDisplay.svelte"

// TODO: Replace with actual contract address and ABI when provided
const QUICK_WITHDRAW_CONTRACT_ADDRESS = "0x0000000000000000000000000000000000000000" as const
const QUICK_WITHDRAW_ABI = [] as const // Will be provided by user

// TODO: When ABI is provided, update the following:
// 1. Active check - read "active" or similar function from contract (line ~144)
// 2. Fixed exchange rate - get the fixed eU to U conversion rate from contract
// 3. Withdraw function - update function name and args (line ~244)
//    - Since it's a fixed rate, no minAmount parameter is needed
//    - Typical args might be just: (uint256 euAmount) or (uint256 euAmount, address recipient)

interface Props {
  evmChain: O.Option<Chain>
  uOnEvmToken: O.Option<TokenType>
  eUOnEvmToken: O.Option<TokenType>
  eUOnEvmBalance: O.Option<bigint>
  onQuickWithdrawSuccess?: () => void
}

let {
  evmChain,
  uOnEvmToken,
  eUOnEvmToken,
  eUOnEvmBalance,
  onQuickWithdrawSuccess,
}: Props = $props()

type QuickWithdrawState = Data.TaggedEnum<{
  Ready: {}
  SwitchingChain: {}
  CheckingAllowance: {}
  ApprovingAllowance: {}
  AllowanceSubmitted: { txHash: string }
  WaitingForAllowanceConfirmation: { txHash: string }
  AllowanceApproved: {}
  ConfirmingWithdraw: {}
  WithdrawSubmitted: { txHash: string }
  WaitingForConfirmation: { txHash: string }
  Success: { txHash: string; receivedAmount: bigint }
  Error: { message: string }
}>

const QuickWithdrawState = Data.taggedEnum<QuickWithdrawState>()

let withdrawInput = $state<string>("")
let withdrawAmount = $state<O.Option<bigint>>(O.none())
let quickWithdrawState = $state<QuickWithdrawState>(QuickWithdrawState.Ready())
let shouldWithdraw = $state<boolean>(false)
let isContractActive = $state<O.Option<boolean>>(O.none())
let shouldCheckActive = $state<boolean>(true)

const isWithdrawing = $derived(
  !QuickWithdrawState.$is("Ready")(quickWithdrawState)
    && !QuickWithdrawState.$is("Success")(quickWithdrawState)
    && !QuickWithdrawState.$is("Error")(quickWithdrawState),
)
const isSuccess = $derived(QuickWithdrawState.$is("Success")(quickWithdrawState))
const isError = $derived(QuickWithdrawState.$is("Error")(quickWithdrawState))

// Derived state for button disabled logic
const isButtonDisabled = $derived(
  pipe(
    O.all([WalletStore.evmAddress, withdrawAmount, eUOnEvmBalance]),
    O.match({
      onNone: () => isWithdrawing || pipe(
        isContractActive,
        O.match({
          onNone: () => false,
          onSome: (active) => !active,
        }),
      ),
      onSome: ([_, amount, balance]) => {
        return isWithdrawing
          || amount === 0n
          || amount > balance
          || pipe(
            isContractActive,
            O.match({
              onNone: () => false,
              onSome: (active) => !active,
            }),
          )
      },
    }),
  ),
)

const inputAmount = $derived<O.Option<BigDecimal.BigDecimal>>(pipe(
  withdrawInput,
  BigDecimal.fromString,
))

// TODO: Get actual fixed exchange rate from contract
const exchangeRate = $state<BigDecimal.BigDecimal>(BigDecimal.fromBigInt(1n)) // Placeholder 1:1 rate

// Calculate exact U output based on fixed exchange rate (no slippage needed)
const expectedUAmount = $derived<O.Option<BigDecimal.BigDecimal>>(pipe(
  inputAmount,
  O.map(amount => {
    // Apply fixed exchange rate
    return BigDecimal.multiply(amount, exchangeRate)
  }),
))

// Check if the quick withdraw contract is active
runPromiseExit$(() =>
  shouldCheckActive && O.isSome(WalletStore.evmAddress)
    ? Effect.gen(function*() {
      const connectorClient = yield* getWagmiConnectorClient
      
      const publicClient = Evm.PublicClient.Live({
        chain: mainnet,
        transport: custom(connectorClient),
      })

      // TODO: Update with actual active check when ABI is provided
      // For now, return true as placeholder
      // When ABI is provided, use:
      // const active = yield* Evm.readContract({
      //   address: QUICK_WITHDRAW_CONTRACT_ADDRESS,
      //   abi: QUICK_WITHDRAW_ABI,
      //   functionName: "active",
      //   args: [],
      // }).pipe(Effect.provide(publicClient))
      
      const active = true // Placeholder - will read from contract
      
      isContractActive = O.some(active)
      shouldCheckActive = false
      return active
    }).pipe(
      Effect.provide(Evm.PublicClient.Live({
        chain: mainnet, 
        transport: custom({ request: async () => ({}) } as any),
      })),
      Effect.catchAll(() =>
        Effect.gen(function*() {
          isContractActive = O.some(false)
          shouldCheckActive = false
          return yield* Effect.succeed(false)
        })
      ),
    )
    : Effect.void
)

// Check and approve eU token allowance for the quick withdraw contract
// The contract needs permission to spend the user's eU tokens
// In exchange, the contract will send U tokens from its pool back to the user at a fixed rate
const checkAndSubmitAllowance = (sender: `0x${string}`, sendAmount: bigint) =>
  pipe(
    Evm.readErc20Allowance(
      EU_ERC20.address,  // Checking allowance for eU tokens (what user sends)
      sender,
      QUICK_WITHDRAW_CONTRACT_ADDRESS,  // Contract that will spend the eU
    ),
    Effect.tap(() =>
      Effect.sync(() => {
        quickWithdrawState = QuickWithdrawState.CheckingAllowance()
      })
    ),
    Effect.flatMap((amount) =>
      Effect.if(amount < sendAmount, {
        onTrue: () =>
          pipe(
            Effect.log(`Approving ${sendAmount} eU for quick withdraw contract`),
            Effect.andThen(() =>
              Effect.sync(() => {
                quickWithdrawState = QuickWithdrawState.ApprovingAllowance()
              })
            ),
            Effect.andThen(() =>
              pipe(
                Evm.increaseErc20Allowance(
      EU_ERC20.address,
      Ucs05.EvmDisplay.make({ address: QUICK_WITHDRAW_CONTRACT_ADDRESS }),
                  sendAmount,
                ),
                Effect.tap((hash) =>
                  Effect.sync(() => {
                    quickWithdrawState = QuickWithdrawState.AllowanceSubmitted({ txHash: hash })
                  })
                ),
                Effect.tap(() => Effect.sleep("500 millis")),
                Effect.tap((hash) =>
                  Effect.sync(() => {
                    quickWithdrawState = QuickWithdrawState.WaitingForAllowanceConfirmation({
                      txHash: hash,
                    })
                  })
                ),
                Effect.andThen((hash) => Evm.waitForTransactionReceipt(hash as `0x${string}`)),
              )
            ),
          ),
        onFalse: () => Effect.log(`eU allowance already sufficient for quick withdraw`),
      })
    ),
    Effect.tap(() =>
      Effect.sync(() => {
        quickWithdrawState = QuickWithdrawState.AllowanceApproved()
      })
    ),
    Effect.tap(() => Effect.sleep("500 millis")),
  )

const executeQuickWithdraw = (sender: `0x${string}`, sendAmount: bigint) =>
  Effect.gen(function*() {
    quickWithdrawState = QuickWithdrawState.ConfirmingWithdraw()

    // Get expected U amount for display (fixed rate, no slippage)
    const expectedAmount = O.match(expectedUAmount, {
      onNone: () => 0n,
      onSome: (amount) => Utils.toRawAmount(amount),
    })

    // Step 2: Execute the withdraw transaction at fixed rate
    // User sends eU to the contract, contract sends U back from its pool at a fixed rate
    // TODO: Update with actual function name and arguments when ABI is provided
    // Typical pattern might be: withdraw(uint256 euAmount) or withdraw(uint256 euAmount, address recipient)
    const txHash = yield* Evm.writeContract({
      address: QUICK_WITHDRAW_CONTRACT_ADDRESS,
      abi: QUICK_WITHDRAW_ABI,
      functionName: "withdraw", // Placeholder - update with actual function name
      args: [sendAmount], // Update args based on actual contract (no minAmount needed for fixed rate)
      account: sender,
      chain: mainnet,
    })

    quickWithdrawState = QuickWithdrawState.WithdrawSubmitted({ txHash })
    yield* Effect.sleep("500 millis")

    quickWithdrawState = QuickWithdrawState.WaitingForConfirmation({ txHash })

    const receipt = yield* Evm.waitForTransactionReceipt(txHash)

    // TODO: Parse actual received amount from receipt logs if needed
    const receivedAmount = expectedAmount // At fixed rate, received = expected

    quickWithdrawState = QuickWithdrawState.Success({ txHash, receivedAmount })

    return { txHash, receivedAmount }
  })

runPromiseExit$(() =>
  shouldWithdraw
    ? Effect.gen(function*() {
      const validatedData = O.all({
        sender: WalletStore.evmAddress,
        sendAmount: withdrawAmount,
        chain: evmChain,
      })

      if (O.isNone(validatedData)) {
        quickWithdrawState = QuickWithdrawState.Error({
          message: "Missing required data: wallet address, withdraw amount, or chain",
        })
        shouldWithdraw = false
        return yield* Effect.fail(new Error("Missing required data"))
      }

      const { sender, sendAmount, chain } = validatedData.value

      quickWithdrawState = QuickWithdrawState.SwitchingChain()

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

      yield* checkAndSubmitAllowance(sender.address, sendAmount).pipe(
        Effect.provide(walletClient),
        Effect.provide(publicClient),
        Effect.provide(maybeSafe),
      )

      const { txHash, receivedAmount } = yield* executeQuickWithdraw(sender.address, sendAmount).pipe(
        Effect.provide(walletClient),
        Effect.provide(publicClient),
        Effect.provide(maybeSafe),
      )

      withdrawInput = ""
      shouldWithdraw = false
      onQuickWithdrawSuccess?.()

      setTimeout(() => {
        if (QuickWithdrawState.$is("Success")(quickWithdrawState)) {
          quickWithdrawState = QuickWithdrawState.Ready()
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

          quickWithdrawState = QuickWithdrawState.Error({ message: shortMessage })
          shouldWithdraw = false
          return yield* Effect.void
        })
      ),
    )
    : Effect.void
)

function handleButtonClick() {
  if (isWithdrawing) {
    return
  }

  Match.value({ isError, isSuccess, hasWallet: O.isSome(WalletStore.evmAddress) }).pipe(
    Match.when({ isError: true }, () => {
      quickWithdrawState = QuickWithdrawState.Ready()
    }),
    Match.when({ isSuccess: true }, () => {
      quickWithdrawState = QuickWithdrawState.Ready()
    }),
    Match.when({ hasWallet: false }, () => {
      uiStore.openWalletModal()
    }),
    Match.orElse(() => {
      quickWithdrawState = QuickWithdrawState.Ready()
      shouldWithdraw = true
    }),
  )
}
</script>

<div class="flex grow flex-col gap-4">
  <!-- Input Section with Balance -->
  <div class="space-y-3">
    <div class="flex justify-between items-center">
      <label
        for="withdrawInput"
        class="text-xs font-medium text-zinc-400 uppercase tracking-wider"
      >Amount to Quick Withdraw</label>
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
        id="withdrawInput"
        label=""
        placeholder="0.0"
        disabled={O.isNone(eUOnEvmBalance)}
        token={eUOnEvmToken}
        balance={eUOnEvmBalance}
        bind:humanValue={withdrawInput}
        bind:weiValue={withdrawAmount}
      />

      <!-- Quick Percentage Buttons -->
      <div class="mt-2">
        <QuickAmountButtons
          balance={eUOnEvmBalance}
          decimals={18}
          onAmountSelect={(amount, wei) => {
            withdrawInput = amount
            withdrawAmount = O.some(wei)
          }}
        />
      </div>
    </div>
  </div>

  <div class="flex-1"></div>

  <!-- Transaction Preview Card -->
  <div class="rounded-lg bg-zinc-900 border border-zinc-800/50 p-3 space-y-3">
    <!-- Exchange Rate -->
    <div class="flex justify-between items-center">
      <span class="text-xs text-zinc-500">Exchange Rate</span>
      <span class="text-sm font-medium text-zinc-200">
        1 eU = {
          pipe(
            exchangeRate,
            BigDecimal.round({ mode: "from-zero", scale: 6 }),
            Utils.formatBigDecimal,
          )
        } U
      </span>
    </div>

    <!-- Fixed Rate Info -->
    <div class="flex justify-between items-center">
      <span class="text-xs text-zinc-500">Rate Type</span>
      <span class="text-sm font-medium text-zinc-200">Fixed (no slippage)</span>
    </div>

    <!-- Instant Withdrawal Info -->
    <div class="flex justify-between items-center">
      <span class="text-xs text-zinc-500">Withdrawal Type</span>
      <span class="text-sm font-medium text-zinc-200">Instant (no waiting)</span>
    </div>

    <!-- You'll Receive -->
    <div class="pt-2 border-t border-zinc-800">
      <div class="flex justify-between items-center">
        <span class="text-xs text-zinc-500">You'll Receive</span>
        <div class="text-right">
          {#if O.isSome(evmChain) && O.isSome(uOnEvmToken) && O.isSome(expectedUAmount)}
            <TokenComponent
              chain={evmChain.value}
              denom={uOnEvmToken.value.denom}
              amount={TokenRawAmount.make(Utils.toRawAmount(expectedUAmount.value))}
              showWrapping={false}
              showSymbol={true}
              showIcon={true}
              maxDecimals={4}
            />
          {:else}
            <span class="text-zinc-300">
              {
                pipe(
                  expectedUAmount,
                  O.map(bd => Utils.formatBigDecimal(bd)),
                  O.getOrElse(() => "0"),
                )
              } U
            </span>
          {/if}
        </div>
      </div>
    </div>
  </div>

  <!-- Contract Status Warning (only show if not active) -->
  {#if pipe(isContractActive, O.match({ onNone: () => false, onSome: (active) => !active }))}
    <div class="rounded-lg bg-orange-500/10 border border-orange-500/20 p-3">
      <div class="flex items-start gap-2">
        <svg
          class="w-4 h-4 text-orange-400 mt-0.5 flex-shrink-0"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
          />
        </svg>
        <div class="text-xs text-zinc-400">
          Quick withdraw is currently not active. Please use the regular withdraw process or check back later.
        </div>
      </div>
    </div>
  {/if}

  <!-- Status Display -->
  <StatusDisplay
    state={quickWithdrawState}
    type="quick-withdraw"
    inputAmount={withdrawInput}
  />

  <!-- Action Button -->
  <Button
    variant={isError ? "secondary" : "primary"}
    disabled={isButtonDisabled}
    onclick={handleButtonClick}
  >
    {#if isWithdrawing}
      <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin mr-2"></div>
    {/if}
    {
      Match.value(quickWithdrawState).pipe(
        Match.when(QuickWithdrawState.$is("Ready"), () =>
          pipe(
            isContractActive,
            O.match({
              onNone: () => O.isNone(WalletStore.evmAddress)
                ? "Connect Wallet"
                : withdrawInput
                ? "Quick Withdraw"
                : "Enter Amount",
              onSome: (active) => !active
                ? "Quick Withdraw Not Active"
                : O.isNone(WalletStore.evmAddress)
                ? "Connect Wallet"
                : withdrawInput
                ? "Quick Withdraw"
                : "Enter Amount",
            }),
          )),
        Match.when(QuickWithdrawState.$is("SwitchingChain"), () => "Switching..."),
        Match.when(QuickWithdrawState.$is("CheckingAllowance"), () => "Checking..."),
        Match.when(QuickWithdrawState.$is("ApprovingAllowance"), () => "Approve in Wallet"),
        Match.when(QuickWithdrawState.$is("AllowanceSubmitted"), () => "Processing..."),
        Match.when(QuickWithdrawState.$is("WaitingForAllowanceConfirmation"), () => "Confirming..."),
        Match.when(QuickWithdrawState.$is("AllowanceApproved"), () => "Approved ✓"),
        Match.when(QuickWithdrawState.$is("ConfirmingWithdraw"), () => "Confirm in Wallet"),
        Match.when(QuickWithdrawState.$is("WithdrawSubmitted"), () => "Processing..."),
        Match.when(QuickWithdrawState.$is("WaitingForConfirmation"), () => "Confirming..."),
        Match.when(QuickWithdrawState.$is("Success"), () => "Withdraw Again"),
        Match.when(QuickWithdrawState.$is("Error"), () => "Try Again"),
        Match.exhaustive,
      )
    }
  </Button>
</div>
