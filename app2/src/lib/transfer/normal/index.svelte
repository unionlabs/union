<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import StepProgressBar from "$lib/components/ui/StepProgressBar.svelte"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import {
  FillingStep,
  ApprovalStep,
  SubmitStep,
  IndexStep,
  CheckReceiverStep,
  Steps
} from '$lib/transfer/normal/steps';
import { Array as Arr, Effect, Fiber, FiberId, Option } from "effect"
import {
  CreateContextState,
  createContextState,
  type StateResult
} from "$lib/transfer/shared/services/filling/index.ts"
import type { ContextFlowError } from "$lib/transfer/shared/errors"
import { transferHashStore } from "$lib/stores/transfer-hash.svelte.ts"
import { constVoid, pipe } from "effect/Function"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import { beforeNavigate } from "$app/navigation"
import { onMount } from "svelte"
import { fly } from "svelte/transition"
import type { TransferContext } from "$lib/transfer/shared/services/filling/create-context.ts"

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

function goToPreviousPage() {
  if (currentPage > 0) {
    previousPage = currentPage
    currentPage--
  }
}

let actionButtonText = $derived.by(() => {
  if (Option.isNone(transferSteps)) return "Submit"
  const steps = transferSteps.value
  if (currentPage < 0 || currentPage >= steps.length || !steps[currentPage]) return "Submit"
  const currentStep = steps[currentPage]
  if (currentPage === steps.length - 1) return "Complete"
  return Steps.match(currentStep, {
    Filling: () => "Continue",
    CheckReceiver: () => "Continue",
    ApprovalRequired: () => "Approve",
    SubmitInstruction: () => "Submit",
    WaitForIndex: () => "Submit"
  })
})

function handleActionButtonClick() {
  if (Option.isNone(transferSteps)) return
  const currentStep = transferSteps.value[currentPage]

  if (Steps.is("Filling")(currentStep)) {
    goToNextPage()
    return
  }

  if (Steps.is("CheckReceiver")(currentStep)) goToNextPage()
  if (Steps.is("ApprovalRequired")(currentStep)) goToNextPage()
  if (Steps.is("SubmitInstruction")(currentStep)) goToNextPage()
}

function interruptFiber() {
  Option.match(currentFiber, {
    onNone: constVoid,
    onSome: fiber => Fiber.interruptFork(fiber)
  })
  currentFiber = Option.none()
}

function newTransfer() {
  interruptFiber()
  transferSteps = Option.none()
  transferErrors = Option.none()
  isLoading = false
  statusMessage = ""
  currentPage = 0
  transferData.raw.reset()
  transferHashStore.reset()
  wallets.clearInputAddress()
}

$effect(() => {
  if (currentPage !== 0) return
  interruptFiber()

  isLoading = true
  transferSteps = Option.none()
  transferErrors = Option.none()

  const machineEffect = Effect.gen(function* () {
    let currentState: CreateContextState = CreateContextState.Filling()
    let context: TransferContext

    while (true) {
      const result: StateResult = yield* createContextState(currentState, transferData)
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

    const isReceiverInWallet = pipe(
      Option.all({
        destinationChain: transferData.destinationChain,
        receiver: transferData.derivedReceiver
      }),
      Option.flatMap(({ destinationChain, receiver }) => {
        const walletaddr = wallets.getAddressForChain(destinationChain)
        return Option.map(walletaddr, x => x.toLowerCase() === receiver.toLowerCase())
      }),
      Option.getOrElse(() => false)
    )

    if (!isReceiverInWallet) {
      steps.push(
        Steps.CheckReceiver({
          receiver: transferData.derivedReceiver,
          destinationChain: transferData.destinationChain
        })
      )
    }

    if (context) {
      if (Option.isSome(context.allowances)) {
        const allowances = context.allowances.value

        for (let i = 0; i < allowances.length; i++) {
          const allowance = allowances[i]
          const intent = context.intents[i]

          steps.push(
            Steps.ApprovalRequired({
              token: allowance.token,
              requiredAmount: allowance.requiredAmount,
              currentAllowance: allowance.currentAllowance,
              intent
            })
          )
        }
      }

      if (Option.isSome(context.instruction)) {
        const instruction = context.instruction.value

        for (const intent of context.intents) {
          steps.push(
            Steps.SubmitInstruction({ instruction, intent }),
            Steps.WaitForIndex({ intent })
          )
        }
      }
    }

    transferSteps = Option.some(steps)
    isLoading = false
    currentFiber = Option.none()
  })

  const fiber = Effect.runFork(machineEffect as Effect.Effect<void, never, never>)
  currentFiber = Option.some(fiber)

  return () => fiber?.unsafeInterruptAsFork(FiberId.none)
})

beforeNavigate(newTransfer)

onMount(() => {
  const handler = (e: KeyboardEvent) => {
    const metaOrCtrl = e.metaKey || e.ctrlKey

    if (metaOrCtrl && e.altKey && e.shiftKey && e.code === "KeyD") {
      e.preventDefault()
      showDetails = !showDetails
    }
  }

  window.addEventListener("keydown", handler)
  return () => window.removeEventListener("keydown", handler)
})

const currentStep = $derived(
  pipe(
    transferSteps, //[currentPage]
    Option.flatMap(Arr.get(currentPage)),
    Option.getOrElse(() => Steps.Filling())
  )
)
</script>

<Card
  divided
  class="max-w-sm w-full mt-24 relative self-center flex flex-col justify-between min-h-[450px] overflow-hidden transition-transform duration-500"
>
  <div class="w-full">
    <StepProgressBar
      class="w-full"
      currentStep={currentPage + 1}
      totalSteps={transferSteps.pipe(
        Option.map(ts => ts.length),
        Option.getOrElse(() => 1)
      )}
      stepDescriptions={transferSteps.pipe(
        Option.map(ts => ts.map(Steps.description)),
        Option.getOrElse(() => ["Configure your transfer"])
      )}
    />
  </div>

  <div class="grid w-full grow overflow-hidden">
    {#if currentPage === 0}
      <div
        class="flex grow col-start-1 col-end-2 row-start-1 row-end-2"
        out:fly={{ x: direction * -382, duration:300 }}
        in:fly ={{ x: direction * 382, duration:300 }}
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
            out:fly={{ x: direction * -382, duration:300 }}
            in:fly ={{ x: direction * 382, duration:300 }}
          >
            <CheckReceiverStep
              stepIndex={currentPage + 1}
              step={currentStep}
              onBack={goToPreviousPage}
              onSubmit={goToNextPage}
            />
          </div>
        {/if}
        {#if Steps.is("ApprovalRequired")(currentStep)}
          <div
            class="flex grow col-start-1 col-end-2 row-start-1 row-end-2"
            out:fly={{ x: direction * -382, duration:300 }}
            in:fly ={{ x: direction * 382, duration:300 }}
          >
            <ApprovalStep
              stepIndex={currentPage + 1}
              step={currentStep}
              onBack={goToPreviousPage}
              onApprove={handleActionButtonClick}
              {actionButtonText}
            />
          </div>
        {/if}
        {#if Steps.is("SubmitInstruction")(currentStep)}
          <div
            class="flex grow col-start-1 col-end-2 row-start-1 row-end-2"
            out:fly={{ x: direction * -382, duration:300 }}
            in:fly ={{ x: direction * 382, duration:300 }}
          >
            <SubmitStep
              stepIndex={currentPage + 1}
              step={currentStep}
              onCancel={newTransfer}
              onSubmit={handleActionButtonClick}
              {actionButtonText}
            />
          </div>
        {/if}
        {#if Steps.is("WaitForIndex")(currentStep)}
          <div
            class="flex grow col-start-1 col-end-2 row-start-1 row-end-2"
            out:fly={{ x: direction * -382, duration:300 }}
            in:fly ={{ x: direction * 382, duration:300 }}
          >
            <IndexStep {newTransfer} step={currentStep} />
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

