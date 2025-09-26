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
import { Ucs05, Utils } from "@unionlabs/sdk"
import { Evm, Safe } from "@unionlabs/sdk-evm"
import { EU_ERC20 } from "@unionlabs/sdk/Constants"
import type { Chain, Token as TokenType } from "@unionlabs/sdk/schema"
import { TokenRawAmount } from "@unionlabs/sdk/schema"
import { extractErrorDetails } from "@unionlabs/sdk/utils/index"
import { BigDecimal, Data, Effect, Layer, Match, pipe } from "effect"
import * as O from "effect/Option"
import { createPublicClient, custom, http } from "viem"
import { mainnet } from "viem/chains"
import QuickAmountButtons from "./QuickAmountButtons.svelte"
import StatusDisplay from "./StatusDisplay.svelte"

const QUICK_WITHDRAW_CONTRACT_ADDRESS = "0xFADE236fAa8c35D721Aa01480497A07e23A29d19" as const
const QUICK_WITHDRAW_ABI = [
  {
    name: "active",
    type: "function",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "bool" }],
  },
  {
    inputs: [{ name: "baseAmount", type: "uint256" }],
    name: "withdraw",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
] as const

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

const isButtonDisabled = $derived(
  pipe(
    O.all([WalletStore.evmAddress, withdrawAmount, eUOnEvmBalance]),
    O.match({
      onNone: () =>
        isWithdrawing || pipe(
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

const exchangeRate = $state<BigDecimal.BigDecimal>(BigDecimal.make(9n, 1)) // 0.9 rate

const expectedUAmount = $derived<O.Option<BigDecimal.BigDecimal>>(pipe(
  inputAmount,
  O.map(amount => BigDecimal.multiply(amount, exchangeRate)),
))

runPromiseExit$(() =>
  shouldCheckActive
    ? Effect.gen(function*() {
      const publicClient = createPublicClient({
        chain: mainnet,
        transport: http("https://rpc.1.ethereum.chain.kitchen"),
      })

      const active = yield* pipe(
        Effect.tryPromise({
          try: () =>
            publicClient.readContract({
              address: QUICK_WITHDRAW_CONTRACT_ADDRESS,
              abi: QUICK_WITHDRAW_ABI,
              functionName: "active",
              args: [],
            }),
          catch: (error) => new Error(`Failed to check active status: ${error}`),
        }),
        Effect.tap(result => Effect.log(`Contract active: ${result}`)),
        Effect.catchAll(error =>
          Effect.gen(function*() {
            yield* Effect.logError(error)
            return false
          })
        ),
      )

      isContractActive = O.some(active)
      shouldCheckActive = false
      return active
    }).pipe(
      Effect.catchAll((error) =>
        Effect.gen(function*() {
          yield* Effect.logError("Contract status check failed", error)
          shouldCheckActive = false
          isContractActive = O.some(false)
          return yield* Effect.succeed(false)
        })
      ),
    )
    : Effect.void
)

const checkAndSubmitAllowance = (sender: `0x${string}`, sendAmount: bigint) =>
  pipe(
    Evm.readErc20Allowance(EU_ERC20.address, sender, QUICK_WITHDRAW_CONTRACT_ADDRESS),
    Effect.tap(() =>
      Effect.sync(() => quickWithdrawState = QuickWithdrawState.CheckingAllowance())
    ),
    Effect.flatMap((amount) =>
      Effect.if(amount < sendAmount, {
        onTrue: () =>
          pipe(
            Effect.log(`Insufficient allowance: ${amount} < ${sendAmount}, requesting approval`),
            Effect.andThen(() =>
              Effect.sync(() => quickWithdrawState = QuickWithdrawState.ApprovingAllowance())
            ),
            Effect.andThen(() =>
              pipe(
                Evm.increaseErc20Allowance(
                  EU_ERC20.address,
                  Ucs05.EvmDisplay.make({ address: QUICK_WITHDRAW_CONTRACT_ADDRESS }),
                  sendAmount,
                ),
                Effect.tap((hash) => {
                  quickWithdrawState = QuickWithdrawState.AllowanceSubmitted({ txHash: hash })
                  return Effect.log(`Approval tx: ${hash}`)
                }),
                Effect.tap(() => Effect.sleep("500 millis")),
                Effect.tap((hash) =>
                  Effect.sync(() =>
                    quickWithdrawState = QuickWithdrawState.WaitingForAllowanceConfirmation({
                      txHash: hash,
                    })
                  )
                ),
                Effect.andThen((hash) =>
                  Evm.waitForTransactionReceipt(hash as `0x${string}`)
                    .pipe(Effect.timeout("120 seconds"))
                ),
              )
            ),
          ),
        onFalse: () => Effect.log(`Allowance sufficient: ${amount} >= ${sendAmount}`),
      })
    ),
    Effect.tap(() =>
      Effect.sync(() => quickWithdrawState = QuickWithdrawState.AllowanceApproved())
    ),
    Effect.tap(() => Effect.sleep("500 millis")),
  )

const executeQuickWithdraw = (sender: `0x${string}`, sendAmount: bigint) =>
  Effect.gen(function*() {
    quickWithdrawState = QuickWithdrawState.ConfirmingWithdraw()

    const expectedAmount = O.match(expectedUAmount, {
      onNone: () => 0n,
      onSome: (amount) => Utils.toRawAmount(amount),
    })

    yield* Effect.log(`Executing withdraw: ${sendAmount} eU for ~${expectedAmount} U`)
    const txHash = yield* Evm.writeContract({
      address: QUICK_WITHDRAW_CONTRACT_ADDRESS,
      abi: QUICK_WITHDRAW_ABI,
      functionName: "withdraw",
      args: [sendAmount],
      account: sender,
      chain: mainnet,
    })

    quickWithdrawState = QuickWithdrawState.WithdrawSubmitted({ txHash })
    yield* Effect.sleep("500 millis")

    quickWithdrawState = QuickWithdrawState.WaitingForConfirmation({ txHash })

    const receipt = yield* Evm.waitForTransactionReceipt(txHash).pipe(
      Effect.timeout("180 seconds"),
    )
    yield* Effect.log(`Transaction confirmed: ${txHash}`)

    const receivedAmount = expectedAmount

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
        Effect.tapError((error) => Effect.logError("Approval flow failed", error)),
      )

      const { txHash, receivedAmount } = yield* executeQuickWithdraw(sender.address, sendAmount)
        .pipe(
          Effect.provide(walletClient),
          Effect.provide(publicClient),
          Effect.provide(maybeSafe),
          Effect.tapError((error) => Effect.logError("Exit flow failed", error)),
        )

      withdrawInput = ""
      shouldWithdraw = false
      onQuickWithdrawSuccess?.()

      yield* Effect.sleep("5 seconds")
      if (QuickWithdrawState.$is("Success")(quickWithdrawState)) {
        quickWithdrawState = QuickWithdrawState.Ready()
      }
    }).pipe(
      Effect.catchAll(error =>
        Effect.gen(function*() {
          const errorDetails = extractErrorDetails(error)
          const message = String(errorDetails?.message || error).split(".")[0]

          yield* Effect.logError("Quick withdraw failed", { error: errorDetails, message })
          quickWithdrawState = QuickWithdrawState.Error({ message })
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
  <div class="space-y-3">
    <div class="flex justify-between items-center">
      <label
        for="withdrawInput"
        class="text-xs font-medium text-zinc-400 uppercase tracking-wider"
      >Amount</label>
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

  <div class="rounded-lg bg-zinc-900 border border-zinc-800/50 p-3 space-y-3">
    <div class="text-xs text-zinc-400">
      Instant Exit lets you withdraw your staked U for a 10% slashing fee. You will exit your
      staking position and not receive staking rewards. This feature is intended for pre-stakers."
    </div>

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

  <StatusDisplay
    state={quickWithdrawState}
    type="quick-withdraw"
    inputAmount={withdrawInput}
    isContractActive={isContractActive}
  />

  <Button
    variant={isError ? "secondary" : "danger"}
    disabled={isButtonDisabled}
    onclick={handleButtonClick}
  >
    {#if isWithdrawing}
      <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin mr-2">
      </div>
    {/if}
    {
      Match.value(quickWithdrawState).pipe(
        Match.when(QuickWithdrawState.$is("Ready"), () =>
          pipe(
            isContractActive,
            O.match({
              onNone: () =>
                O.isNone(WalletStore.evmAddress)
                  ? "Connect Wallet"
                  : withdrawInput
                  ? "I understand, exit now."
                  : "Enter Amount",
              onSome: (active) =>
                !active
                  ? "Instant Exit Not Active"
                  : O.isNone(WalletStore.evmAddress)
                  ? "Connect Wallet"
                  : withdrawInput
                  ? "I understand, exit now."
                  : "Enter Amount",
            }),
          )),
        Match.when(QuickWithdrawState.$is("SwitchingChain"), () => "Switching..."),
        Match.when(QuickWithdrawState.$is("CheckingAllowance"), () => "Checking..."),
        Match.when(QuickWithdrawState.$is("ApprovingAllowance"), () => "Approve in Wallet"),
        Match.when(QuickWithdrawState.$is("AllowanceSubmitted"), () => "Processing..."),
        Match.when(QuickWithdrawState.$is("WaitingForAllowanceConfirmation"), () =>
          "Confirming..."),
        Match.when(QuickWithdrawState.$is("AllowanceApproved"), () =>
          "Approved ✓"),
        Match.when(QuickWithdrawState.$is("ConfirmingWithdraw"), () =>
          "Confirm Exit in Wallet"),
        Match.when(QuickWithdrawState.$is("WithdrawSubmitted"), () =>
          "Processing..."),
        Match.when(QuickWithdrawState.$is("WaitingForConfirmation"), () =>
          "Confirming..."),
        Match.when(QuickWithdrawState.$is("Success"), () => "Exit Again"),
        Match.when(QuickWithdrawState.$is("Error"), () => "Try Again"),
        Match.exhaustive,
      )
    }
  </Button>
</div>
