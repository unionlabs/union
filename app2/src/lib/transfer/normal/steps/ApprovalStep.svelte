<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import InsetError from "$lib/components/model/InsetError.svelte"
import Label from "$lib/components/ui/Label.svelte"
import Input from "$lib/components/ui/Input.svelte"
import { Cause, Effect, Exit, Match, Option } from "effect"
import { createViemPublicClient } from "@unionlabs/sdk/evm"
import { getWalletClient } from "$lib/services/evm/clients.ts"
import { getCosmWasmClient } from "$lib/services/cosmos/clients.ts"
import {
  TransactionSubmissionEvm,
  nextStateEvm,
  isComplete as evmIsComplete,
  hasFailedExit as evmHasFailedExit
} from "$lib/transfer/shared/services/write-evm.ts"
import {
  TransactionSubmissionCosmos,
  nextStateCosmos,
  isComplete as cosmosIsComplete,
  hasFailedExit as cosmosHasFailedExit
} from "$lib/transfer/shared/services/write-cosmos.ts"
import { cosmosStore } from "$lib/wallet/cosmos"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import { erc20Abi, http, isHex, toHex } from "viem"
import type { Steps } from "$lib/transfer/normal/steps"
import { constVoid } from "effect/Function"

//Probably something we can import from somewhere?
const MAX_UINT256 = BigInt("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
const MAX_UINT128 = BigInt("340282366920938463463374607431768211455")

type Props = {
  stepIndex: number
  step: Steps.ApprovalRequired
  onBack: () => void
  onApprove: () => void
  actionButtonText: string
}

const { step, onBack, onApprove, actionButtonText }: Props = $props()

let ets = $state<TransactionSubmissionEvm>(TransactionSubmissionEvm.Filling())
let cts = $state<TransactionSubmissionCosmos>(TransactionSubmissionCosmos.Filling())

let showError = $state(false)
let isSubmitting = $state(false)
let error = $state<Option.Option<unknown>>(Option.none())
let selectedMultiplier = $state<1 | "max" | null>(1)
let customAmount = $state("")
let showCustomInput = $state(false)

// Derive validation state
const isValidAmount = $derived(showCustomInput ? isValidCustomAmount(customAmount) : true)

// Derive the actual approval amount
const approvalAmount = $derived(
  selectedMultiplier === "max"
    ? getMaxApprovalAmount()
    : selectedMultiplier === 1
      ? step.requiredAmount
      : customAmount && isValidCustomAmount(customAmount)
        ? Effect.runSync(
            Effect.try({
              try: () => {
                const [whole = "0", fraction = ""] = customAmount.replace(",", ".").split(".")
                const cleanWhole = whole === "0" ? "0" : whole.replace(/^0+/, "")
                const paddedFraction = fraction.padEnd(step.intent.decimals, "0")
                return BigInt(cleanWhole + paddedFraction)
              },
              catch: () => step.requiredAmount
            })
          )
        : step.requiredAmount
)

// Derive button state
const isButtonEnabled = $derived(
  !isSubmitting &&
    ((ets._tag === "Filling" && cts._tag === "Filling") ||
      evmHasFailedExit(ets) ||
      cosmosHasFailedExit(cts))
)

// Derive submit button text
const submitButtonText = $derived(
  ets._tag === "SwitchChainInProgress"
    ? "Switching Chain..."
    : ets._tag === "WriteContractInProgress"
      ? "Confirming Transaction..."
      : ets._tag === "TransactionReceiptInProgress"
        ? "Waiting for Receipt..."
        : cts._tag === "SwitchChainInProgress"
          ? "Switching Chain..."
          : cts._tag === "WriteContractInProgress"
            ? "Confirming Transaction..."
            : evmHasFailedExit(ets) || cosmosHasFailedExit(cts)
              ? "Try Again"
              : actionButtonText
)

const submit = Effect.gen(function* () {
  isSubmitting = true
  error = Option.none()

  // Validate custom amount if in custom input mode
  if (showCustomInput && !(customAmount && isValidCustomAmount(customAmount))) {
    error = Option.some(new Error("Custom amount must be greater than the required amount"))
    isSubmitting = false
    return
  }

  try {
    const chain = step.intent.sourceChain
    const rpcType = chain.rpc_type
    const approvalAmount = getApprovalAmount()

    yield* Match.value(rpcType).pipe(
      Match.when("evm", () =>
        Effect.gen(function* () {
          const viemChain = chain.toViemChain()
          if (Option.isNone(viemChain)) return Effect.succeed(null)

          const publicClient = yield* createViemPublicClient({
            chain: viemChain.value,
            transport: http()
          })

          const walletClient = yield* getWalletClient(chain)

          do {
            ets = yield* Effect.promise(() =>
              nextStateEvm(ets, viemChain.value, publicClient, walletClient, {
                chain: viemChain.value,
                account: walletClient.account,
                address: step.token,
                abi: erc20Abi,
                functionName: "approve",
                args: [step.intent.ucs03address, approvalAmount]
              })
            )

            if (ets._tag === "SwitchChainComplete" || ets._tag === "WriteContractComplete") {
              yield* Exit.matchEffect(ets.exit, {
                onFailure: cause => Effect.sync(() => (error = Option.some(Cause.squash(cause)))),
                onSuccess: () => Effect.sync(() => (error = Option.none()))
              })
            }

            if (evmIsComplete(ets)) {
              onApprove()
              break
            }
          } while (!evmHasFailedExit(ets))

          return Effect.succeed(ets)
        })
      ),
      Match.when("cosmos", () =>
        Effect.gen(function* () {
          console.log("prior to do block")

          const sender = yield* chain.getDisplayAddress(step.intent.sender) // TODO: fix type error

          console.log("before do block")

          do {
            cts = yield* Effect.promise(() =>
              nextStateCosmos(cts, chain, sender, step.token, {
                increase_allowance: {
                  spender: step.intent.sourceChain.minter_address_display,
                  amount: approvalAmount
                }
              })
            )

            if (cts._tag === "SwitchChainComplete" || cts._tag === "WriteContractComplete") {
              yield* Exit.matchEffect(cts.exit, {
                onFailure: cause => Effect.sync(() => (error = Option.some(Cause.squash(cause)))),
                onSuccess: () => Effect.sync(() => (error = Option.none()))
              })
            }

            if (cosmosIsComplete(cts)) {
              onApprove()
              break
            }
          } while (!cosmosHasFailedExit(cts))

          return Effect.succeed(cts)
        })
      ),
      Match.orElse(() =>
        Effect.gen(function* () {
          yield* Effect.log("Unsupported chain type")
          error = Option.some(new Error("Unsupported chain type"))
          return Effect.succeed("unsupported")
        })
      )
    )
  } finally {
    isSubmitting = false
  }
})

const handleSubmit = () => {
  error = Option.none()
  showError = false
  Effect.runPromiseExit(submit).then(exit =>
    Exit.match(exit, {
      onFailure: cause => {
        const err = Cause.originalError(cause)
        console.error("Uncaught approval error:", Cause.pretty(cause))
        error = Option.some(err)
        isSubmitting = false
      },
      onSuccess: constVoid
    })
  )
}

const sourceChain = step.intent.sourceChain
const massagedDenom = isHex(step.token) ? step.token : toHex(step.token)

function getMaxApprovalAmount() {
  return Match.value(step.intent.sourceChain.rpc_type).pipe(
    Match.when("evm", () => MAX_UINT256),
    Match.when("cosmos", () => MAX_UINT128),
    Match.orElse(() => step.requiredAmount) // Fallback to required amount for unknown
  )
}

function getApprovalAmount() {
  return Match.value(selectedMultiplier).pipe(
    Match.when(
      m => m === "max",
      () => getMaxApprovalAmount()
    ),
    Match.when(
      m => m === 1,
      () => step.requiredAmount
    ),
    Match.orElse(() => {
      if (!(customAmount && isValidCustomAmount(customAmount))) {
        return step.requiredAmount
      }
      return Effect.runSync(
        Effect.try({
          try: () => {
            const [whole = "0", fraction = ""] = customAmount.replace(",", ".").split(".")
            const cleanWhole = whole === "0" ? "0" : whole.replace(/^0+/, "")
            const paddedFraction = fraction.padEnd(step.intent.decimals, "0")
            return BigInt(cleanWhole + paddedFraction)
          },
          catch: () => step.requiredAmount
        })
      )
    })
  )
}

function handleMultiplierSelect(multiplier: 1 | "max") {
  selectedMultiplier = multiplier
  if (multiplier === 1) {
    const raw = step.requiredAmount.toString().padStart(step.intent.decimals + 1, "0")
    const whole = raw.slice(0, -step.intent.decimals) || "0"
    const fraction = raw.slice(-step.intent.decimals).replace(/0+$/, "")
    customAmount = fraction ? `${whole}.${fraction}` : whole
  } else if (multiplier === "max") {
    customAmount = getMaxApprovalAmount().toString()
  }
}

function handleCustomInput(event: Event) {
  const input = event.target as HTMLInputElement
  customAmount = input.value
  selectedMultiplier = null
}

function isValidCustomAmount(amount: string): boolean {
  return Effect.runSync(
    Effect.gen(function* () {
      // Handle empty or invalid input
      if (!amount || amount === "." || amount === ",") return false

      const rawAmount = yield* Effect.try({
        try: () => {
          const [whole = "0", fraction = ""] = amount.replace(",", ".").split(".")
          const cleanWhole = whole === "0" ? "0" : whole.replace(/^0+/, "")
          const paddedFraction = fraction.padEnd(step.intent.decimals, "0")
          if (cleanWhole.length > 78) return null
          return BigInt(cleanWhole + paddedFraction)
        },
        catch: () => null
      }).pipe(
        Effect.map(n => n !== null && typeof n === "bigint" && n >= step.requiredAmount),
        Effect.orElse(() => Effect.succeed(false))
      )

      return rawAmount
    })
  )
}

function handleBeforeInput(event: InputEvent) {
  return Effect.runSync(
    Effect.gen(function* () {
      const { inputType, data } = event
      const { value } = event.currentTarget as HTMLInputElement
      const proposed = value + (data ?? "")
      const maxDecimals = step.intent.decimals

      const validShape = yield* Effect.succeed(/^\d*[.,]?\d*$/.test(proposed))
      const validDecimalsDot = yield* Effect.succeed(
        !proposed.includes(".") || proposed.split(".")[1].length <= maxDecimals
      )
      const validDecimalsComma = yield* Effect.succeed(
        !proposed.includes(",") || proposed.split(",")[1].length <= maxDecimals
      )
      const isDelete = yield* Effect.succeed(inputType.startsWith("delete"))
      const validDecimals = yield* Effect.succeed(validDecimalsComma && validDecimalsDot)
      const noDuplicateLeadingZeroes = yield* Effect.succeed(!proposed.startsWith("00"))

      const allow = yield* Effect.succeed(
        isDelete || (validDecimals && validShape && noDuplicateLeadingZeroes)
      )

      if (!allow) {
        event.preventDefault()
      }
    })
  )
}

function handleCustomClick() {
  showCustomInput = true
  selectedMultiplier = null
  customAmount = "" // Reset to empty when switching to custom
}

function handleBackClick() {
  showCustomInput = false
  selectedMultiplier = 1 // Select exact amount when going back
  customAmount = step.requiredAmount.toString() // Set to exact amount
}
</script>

<div class="grow relative min-w-full flex flex-col justify-between h-full">
  <div class="grow flex flex-col gap-2 p-4">
    <h3 class="text-lg font-semibold">
      Approve
      <TokenComponent chain={sourceChain} denom={massagedDenom} showWrapping={false} showIcon={false}/>
    </h3>

    <p class="text-sm text-zinc-400">
      You need to approve Union to send
      <TokenComponent chain={sourceChain} denom={massagedDenom}  showWrapping={false}/>. This is a
      one-time approval for this token.
    </p>

      
    <div class="mt-4">
      <Label class="text-zinc-400 mb-2 block text-sm">Required Approval</Label
      >
      <div class="flex items-center gap-2">
        <TokenComponent
          chain={sourceChain}
          denom={massagedDenom}
          amount={step.requiredAmount}
        />
      </div>
    </div>
  </div>

  <div class="flex flex-col justify-between p-4">
    {#if showCustomInput && !isValidAmount && customAmount}
      <div class="text-sm text-red-500 h-full flex items-center gap-2">
        <span
          >Custom amount must be greater than or equal to the required amount</span
        >
      </div>
    {/if}
    <section>
      <Label class="text-zinc-400 mb-3 block">Select Approval Amount</Label>

      {#if !showCustomInput}
        <div class="flex justify-between">
          <button
            class="flex-1 {selectedMultiplier === 1
              ? 'bg-zinc-800'
              : 'bg-zinc-900'} hover:bg-zinc-800 rounded-l-lg h-10 flex items-center justify-center cursor-pointer"
            onclick={() => handleMultiplierSelect(1)}
            disabled={!isButtonEnabled}
          >
            <span
              class={`uppercase text-xs font-semibold ${selectedMultiplier === 1 ? "text-white" : "text-zinc-400"}`}
              >Exact</span
            >
          </button>

          <button
            class="flex-1 {selectedMultiplier === 'max'
              ? 'bg-zinc-800'
              : 'bg-zinc-900'} hover:bg-zinc-800 h-10 flex items-center justify-center cursor-pointer"
            onclick={() => handleMultiplierSelect("max")}
            disabled={!isButtonEnabled}
          >
            <span
              class={`uppercase text-xs font-semibold ${selectedMultiplier === "max" ? "text-white" : "text-zinc-400"}`}
              >Max</span
            >
          </button>

          <button
            class="flex-1 {selectedMultiplier === null
              ? 'bg-zinc-800'
              : 'bg-zinc-900'} hover:bg-zinc-800 rounded-r-lg h-10 flex items-center justify-center cursor-pointer"
            onclick={handleCustomClick}
            disabled={!isButtonEnabled}
          >
            <span
              class={`uppercase text-xs font-semibold ${selectedMultiplier === null ? "text-white" : "text-zinc-400"}`}
              >Custom</span
            >
          </button>
        </div>
      {:else}
        <div class="flex justify-between gap-4">
          <button
            class="bg-zinc-900 hover:bg-zinc-800 rounded-lg h-10 w-14 flex items-center justify-center cursor-pointer"
            onclick={handleBackClick}
            disabled={!isButtonEnabled}
          >
            <span class="text-zinc-400">←</span>
          </button>
          <div class="flex-1">
            <Input
              type="text"
              required
              disabled={!isButtonEnabled}
              autocorrect="off"
              placeholder="Enter custom amount"
              spellcheck="false"
              autocomplete="off"
              inputmode="decimal"
              value={customAmount}
              oninput={handleCustomInput}
              onbeforeinput={handleBeforeInput}
              class="h-10 text-center text-sm"
              id="custom-amount"
            />
          </div>
        </div>
      {/if}
    </section>
  </div>

  <div class="border-t border-zinc-800 sticky bottom-0 bg-zinc-925">
    <div class="flex justify-between p-4">
      <Button variant="secondary" onclick={onBack} disabled={!isButtonEnabled}>
        Back
      </Button>
      {#if Option.isSome(error)}
        <div class="flex justify-end gap-2">
          <Button variant="danger" onclick={() => (showError = true)}
            >Error</Button
          >
          <Button
            variant="primary"
            onclick={handleSubmit}
            disabled={!isButtonEnabled || (showCustomInput && !isValidAmount)}
          >
            {submitButtonText}
          </Button>
        </div>
      {:else}
        <Button
          variant="primary"
          onclick={handleSubmit}
          disabled={!isButtonEnabled || (showCustomInput && !isValidAmount)}
        >
          {submitButtonText}
        </Button>
      {/if}
    </div>
  </div>

  <InsetError
    open={showError}
    error={Option.isSome(error) ? error.value : null}
    onClose={() => {
      showError = false;
      error = Option.none();
    }}
  />
</div>
