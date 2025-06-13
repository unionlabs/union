<script lang="ts">
import { beforeNavigate } from "$app/navigation"
import Card from "$lib/components/ui/Card.svelte"
import StepProgressBar from "$lib/components/ui/StepProgressBar.svelte"
import { runFork } from "$lib/runtime"
import { FeeStore } from "$lib/stores/fee.svelte"
import { keyboardShortcuts } from "$lib/stores/shortcuts.svelte"
import { transferHashStore } from "$lib/stores/transfer-hash.svelte.ts"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import {
  CreateContextState,
  createContextState,
  type StateResult,
} from "$lib/transfer/normal/services/filling.ts"
import {
  ApprovalStep,
  CheckReceiverStep,
  FillingStep,
  IndexStep,
  Steps,
  SubmitStep,
} from "$lib/transfer/normal/steps"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import type { ContextFlowError } from "$lib/transfer/shared/errors"
import type { TransferContext } from "$lib/transfer/shared/services/filling/create-context.ts"
import { TokenRawAmountFromSelf, TokenRawDenom } from "@unionlabs/sdk/schema"
import { Array as Arr, Effect, Either, Fiber, FiberId, Option, Schema } from "effect"
import { constVoid, pipe } from "effect/Function"
import { onMount, untrack } from "svelte"
import { fly } from "svelte/transition"

let currentPage = $state(0)
let previousPage = $state(0)
let isLoading = $state(false)
let transferSteps = $state<Option.Option<Array<Steps.Steps>>>(Option.none())
let transferErrors = $state<Option.Option<ContextFlowError>>(Option.none())
let currentFiber: Option.Option<Fiber.RuntimeFiber<void, never>> = Option.none()
let statusMessage = $state("")
let showDetails = $state(false)

let direction = $derived(currentPage > previousPage ? 1 : -1)

function goToNextPage() {
  if (Option.isSome(transferSteps) && currentPage < transferSteps.value.length - 1) {
    previousPage = currentPage
    currentPage++
  }
}

// Partial reset of the transfer
function cancelTransfer() {
  interruptFiber()
  currentPage = 0
  transferSteps = Option.none()
  transferErrors = Option.none()
  isLoading = false
  transferHashStore.reset()
}

// Full reset of the transfer
function newTransfer() {
  interruptFiber()
  transferSteps = Option.none()
  transferErrors = Option.none()
  isLoading = false
  currentPage = 0
  transferData.raw.reset()
  transferHashStore.reset()
}

let actionButtonText = $derived.by(() => {
  if (Option.isNone(transferSteps)) {
    return "Submit"
  }
  const steps = transferSteps.value
  if (currentPage < 0 || currentPage >= steps.length || !steps[currentPage]) {
    return "Submit"
  }
  const currentStep = steps[currentPage]
  if (currentPage === steps.length - 1) {
    return "Complete"
  }
  return Steps.match(currentStep, {
    Filling: () => "Continue",
    CheckReceiver: () => "Continue",
    ApprovalRequired: () => "Approve",
    SubmitInstruction: () => "Submit",
    WaitForIndex: () => "Submit",
  })
})

function handleActionButtonClick() {
  if (Option.isNone(transferSteps)) {
    return
  }
  const currentStep = transferSteps.value[currentPage]

  if (Steps.is("Filling")(currentStep)) {
    goToNextPage()
    return
  }

  if (Steps.is("CheckReceiver")(currentStep)) {
    goToNextPage()
  }
  if (Steps.is("ApprovalRequired")(currentStep)) {
    goToNextPage()
  }
  if (Steps.is("SubmitInstruction")(currentStep)) {
    goToNextPage()
  }
}

function interruptFiber() {
  Option.match(currentFiber, {
    onNone: constVoid,
    onSome: fiber => Fiber.interruptFork(fiber),
  })
  currentFiber = Option.none()
}

$effect(() => {
  if (currentPage !== 0) {
    return
  }
  interruptFiber()

  isLoading = true
  transferSteps = Option.none()
  transferErrors = Option.none()
  const feeIntent = untrack(() => Option.getRight(FeeStore.feeIntent))
  console.log("FEE INTENT:", feeIntent.toString())

  const machineEffect = Effect.gen(function*() {
    let currentState: CreateContextState = CreateContextState.Filling()
    let context: TransferContext | null = null

    while (true) {
      const result: StateResult | void = yield* createContextState(
        currentState,
        transferData,
        feeIntent,
      )

      if (!result) {
        break
      }

      statusMessage = result.message

      if (Option.isSome(result.error)) {
        transferErrors = result.error
        transferSteps = Option.none()
        isLoading = false
        currentFiber = Option.none()
        return
      }

      if (Option.isSome(result.nextState)) {
        currentState = result.nextState.value
        continue
      }

      if (Option.isSome(result.context)) {
        context = result.context.value
      }

      break
    }

    const steps: Array<Steps.Steps> = [Steps.Filling()]

    // Check if receiver exists in wallet
    yield* Option.Do.pipe(
      Option.bind("destinationChain", () => transferData.destinationChain),
      Option.bind("receiver", () => transferData.derivedReceiver),
      Option.bind("inWallet", ({ destinationChain, receiver }) => {
        const walletaddr = wallets.getAddressForChain(destinationChain)
        return Option.map(walletaddr, x => x.toLowerCase() === receiver.toLowerCase())
      }),
      Option.match({
        onNone: () => Effect.void,
        onSome: ({ inWallet, destinationChain, receiver }) =>
          Effect.if(inWallet, {
            onFalse: () =>
              Effect.sync(() => {
                steps.push(Steps.CheckReceiver({ receiver, destinationChain }))
              }),
            onTrue: () => Effect.void,
          }),
      }),
    )

    if (context) {
      if (Option.isSome(context.allowances)) {
        const allowances = context.allowances.value

        for (let i = 0; i < allowances.length; i++) {
          const allowance = allowances[i]
          const intent = context.intents[i]

          // TODO: refactor with Struct.evolve and Effect.all
          const token = yield* Schema.decode(TokenRawDenom)(allowance.token)
          const requiredAmount = yield* Schema.decode(TokenRawAmountFromSelf)(
            allowance.requiredAmount,
          )
          const currentAllowance = yield* Schema.decode(TokenRawAmountFromSelf)(
            allowance.currentAllowance,
          )

          steps.push(
            Steps.ApprovalRequired({
              token,
              requiredAmount,
              currentAllowance,
              intent,
            }),
          )
        }
      }

      if (Option.isSome(context.instruction)) {
        const instruction = context.instruction.value

        for (const intent of context.intents) {
          steps.push(
            Steps.SubmitInstruction({ instruction, intent }),
            Steps.WaitForIndex({ intent }),
          )
        }
      }
    }

    transferSteps = Option.some(steps)
    isLoading = false
    currentFiber = Option.none()
  })

  const fiber = runFork(machineEffect as Effect.Effect<void, never, never>)
  currentFiber = Option.some(fiber)

  return () => fiber?.unsafeInterruptAsFork(FiberId.none)
})

beforeNavigate(newTransfer)

onMount(() => {
  keyboardShortcuts.addShortcut(["cmd", "option", "shift", "keyd"], () => {
    showDetails = !showDetails
  })
})

const currentStep = $derived(
  pipe(
    transferSteps, // [currentPage]
    Option.flatMap(Arr.get(currentPage)),
    Option.getOrElse(() => Steps.Filling()),
  ),
)
</script>

<Card
  divided
  class="max-w-sm w-full mt-12 md:mt-24 relative self-center flex flex-col justify-between min-h-[498px] overflow-hidden transition-transform duration-500"
>
  <div class="w-full">
    <StepProgressBar
      class="w-full"
      currentStep={currentPage + 1}
      totalSteps={transferSteps.pipe(
        Option.map(ts => ts.length),
        Option.getOrElse(() => 1),
      )}
      stepDescriptions={transferSteps.pipe(
        Option.map(ts => ts.map(Steps.description)),
        Option.getOrElse(() => ["Configure your transfer"]),
      )}
    />
  </div>

  <div class="grid w-full grow overflow-hidden">
    {#if currentPage === 0}
      <div
        class="flex grow col-start-1 col-end-2 row-start-1 row-end-2"
        out:fly={{ x: direction * -382, duration: 300 }}
        in:fly={{ x: direction * 382, duration: 300 }}
      >
        <FillingStep
          onContinue={handleActionButtonClick}
          {statusMessage}
          {transferErrors}
          onErrorClose={() => {
            transferErrors = Option.none()
          }}
          loading={isLoading}
        />
      </div>
    {/if}
    {#if Steps.is("CheckReceiver")(currentStep)}
      <div
        class="flex grow col-start-1 col-end-2 row-start-1 row-end-2"
        out:fly={{ x: direction * -382, duration: 300 }}
        in:fly={{ x: direction * 382, duration: 300 }}
      >
        <CheckReceiverStep
          stepIndex={currentPage + 1}
          step={currentStep}
          cancel={cancelTransfer}
          onSubmit={goToNextPage}
        />
      </div>
    {/if}
    {#if Steps.is("ApprovalRequired")(currentStep)}
      <div
        class="flex grow col-start-1 col-end-2 row-start-1 row-end-2"
        out:fly={{ x: direction * -382, duration: 300 }}
        in:fly={{ x: direction * 382, duration: 300 }}
      >
        <ApprovalStep
          stepIndex={currentPage + 1}
          step={currentStep}
          cancel={cancelTransfer}
          onApprove={handleActionButtonClick}
          {actionButtonText}
        />
      </div>
    {/if}
    {#if Steps.is("SubmitInstruction")(currentStep)}
      <div
        class="flex grow col-start-1 col-end-2 row-start-1 row-end-2"
        out:fly={{ x: direction * -382, duration: 300 }}
        in:fly={{ x: direction * 382, duration: 300 }}
      >
        <SubmitStep
          stepIndex={currentPage + 1}
          step={currentStep}
          cancel={cancelTransfer}
          onSubmit={handleActionButtonClick}
          {actionButtonText}
        />
      </div>
    {/if}
    {#if Steps.is("WaitForIndex")(currentStep)}
      <div
        class="flex grow col-start-1 col-end-2 row-start-1 row-end-2"
        out:fly={{ x: direction * -382, duration: 300 }}
        in:fly={{ x: direction * 382, duration: 300 }}
      >
        <IndexStep
          newTransfer={cancelTransfer}
          step={currentStep}
        />
      </div>
    {/if}
  </div>
</Card>

{#if showDetails}
  {#if Option.isSome(transferErrors)}
    <strong>Error</strong>
    <pre class="text-wrap">{JSON.stringify(transferErrors.value, null, 2)}</pre>
  {/if}

  {#key statusMessage}
    <strong>{statusMessage}</strong>
    <pre>{JSON.stringify(statusMessage, null, 2)}</pre>
  {/key}

  {#if Option.isSome(transferSteps)}
    <div class="mt-4">
      <strong>Steps:</strong>
      <pre>{JSON.stringify(transferSteps.value, null, 2)}</pre>
    </div>
  {/if}
{/if}
