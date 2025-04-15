<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import StepProgressBar from "$lib/components/ui/StepProgressBar.svelte"
import { LockedTransfer } from "./locked-transfer.ts"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import FillingPage from "./pages/FillingPage.svelte"
import ApprovalPage from "./pages/ApprovalPage.svelte"
import SubmitPage from "./pages/SubmitPage.svelte"
import { lockedTransferStore } from "./locked-transfer.svelte.ts"
import { Array as Arr, Effect, Fiber, FiberId, Option } from "effect"
import * as TransferStep from "./transfer-step.ts"
import IndexPage from "$lib/components/Transfer/pages/IndexPage.svelte"
import {
  CreateTransferState,
  createTransferState,
  type StateResult
} from "$lib/components/Transfer/state/filling/index.ts"
import type { TransferFlowError } from "$lib/components/Transfer/state/errors.ts"
import { transferHashStore } from "$lib/stores/transfer-hash.svelte.ts"
import { constVoid, pipe } from "effect/Function"
import CheckReceiverPage from "./pages/CheckReceiverPage.svelte"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import { beforeNavigate } from "$app/navigation"
import { onMount } from "svelte"
import { fly, slide } from "svelte/transition"
import type { TransferIntents } from "$lib/components/Transfer/state/filling/create-intents.ts"

let currentPage = $state(0)
let isLoading = $state(false)
let transferSteps = $state<Option.Option<Array<TransferStep.TransferStep>>>(Option.none())
let transferErrors = $state<Option.Option<TransferFlowError>>(Option.none())
let currentFiber: Option.Option<Fiber.RuntimeFiber<void, never>> = Option.none()
let statusMessage = $state("")
let showDetails = $state(false)

function goToNextPage() {
  if (Option.isSome(transferSteps) && currentPage < transferSteps.value.length - 1) {
    currentPage++
  }
}

function goToPreviousPage() {
  if (currentPage > 0) {
    currentPage--
    if (currentPage === 0) {
      lockedTransferStore.unlock()
    }
  }
}

let actionButtonText = $derived.by(() => {
  if (Option.isNone(transferSteps)) return "Submit"
  const steps = transferSteps.value
  if (currentPage < 0 || currentPage >= steps.length || !steps[currentPage]) return "Submit"
  const currentStep = steps[currentPage]
  if (currentPage === steps.length - 1) return "Complete"
  return TransferStep.match(currentStep, {
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

  if (TransferStep.is("Filling")(currentStep)) {
    if (Option.isNone(lockedTransferStore.get())) {
      lockedTransferStore.reset()
      const newLockedTransfer = LockedTransfer.fromTransfer(
        transfer.sourceChain,
        transfer.destinationChain,
        transfer.channel,
        transfer.parsedAmount,
        transfer.baseToken,
        transferSteps
      )
      if (Option.isSome(newLockedTransfer)) {
        lockedTransferStore.lock(newLockedTransfer.value)
      } else {
        console.error("Failed to lock transfer values")
        return
      }
    }
    goToNextPage()
    return
  }

  if (TransferStep.is("CheckReceiver")(currentStep)) goToNextPage()
  if (TransferStep.is("ApprovalRequired")(currentStep)) goToNextPage()
  if (TransferStep.is("SubmitInstruction")(currentStep)) goToNextPage()
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
  transfer.raw.reset()
  lockedTransferStore.reset()
  transferHashStore.reset()
}

$effect(() => {
  if (currentPage !== 0) return
  interruptFiber()

  isLoading = true
  transferSteps = Option.none()
  transferErrors = Option.none()

  const machineEffect = Effect.gen(function* () {
    let currentState: CreateTransferState = CreateTransferState.Filling()
    let intents: TransferIntents = []

    while (true) {
      const result: StateResult = yield* createTransferState(currentState, transfer)
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

      if (Option.isSome(result.intents)) {
        intents = result.intents.value
      }

      break
    }

    const steps: Array<TransferStep.TransferStep> = [TransferStep.Filling()]

    const isReceiverInWallet = pipe(
      Option.all({
        destinationChain: transfer.destinationChain,
        receiver: transfer.derivedReceiver
      }),
      Option.flatMap(({ destinationChain, receiver }) => {
        const walletaddr = wallets.getAddressForChain(destinationChain)
        return Option.map(walletaddr, x => x.toLowerCase() === receiver.toLowerCase())
      }),
      Option.getOrElse(() => false)
    )

    if (!isReceiverInWallet) {
      steps.push(
        TransferStep.CheckReceiver({
          receiver: transfer.derivedReceiver,
          destinationChain: transfer.destinationChain
        })
      )
    }

    for (const intent of intents) {
      const allowance = Option.getOrUndefined(intent.allowances)
      if (allowance) {
        steps.push(
          TransferStep.ApprovalRequired({
            token: allowance.token,
            requiredAmount: allowance.requiredAmount,
            currentAllowance: allowance.currentAllowance
          })
        )
      }

      const instruction = Option.getOrUndefined(intent.instructions)
      if (instruction) {
        steps.push(
          TransferStep.SubmitInstruction({
            instruction,
            intents
          }),
          TransferStep.WaitForIndex()
        )
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

$effect(() => {
  console.log("lukas", transferErrors)
})

const currentStepTag = $derived(
  pipe(
    transferSteps,
    Option.flatMap(Arr.get(currentPage)),
    Option.map(x => x._tag),
    Option.getOrElse(() => "Filling" as const)
  )
)

const currentLockedStepTag = $derived(
  pipe(
    lockedTransferStore.get(),
    Option.flatMap(x => x.getStep(currentPage)),
    Option.map(x => x._tag),
    Option.getOrNull
  )
)

$effect(() => {
  console.log({
    currentStepTag,
    currentLockedStepTag
  })
})

const flyLeft = (node: Element) =>
  fly(node, {
    x: -300,
    duration: 1000,
    delay: 100
  })

const flyRight = (node: Element) =>
  fly(node, {
    x: 300,
    duration: 1000,
    delay: 100
  })
</script>

<Card
  divided
  class="max-w-sm w-full mt-24 relative self-center flex flex-col justify-between min-h-[450px] overflow-hidden transition-transform duration-500"
>
  <div class="w-full">
    <StepProgressBar
      class="w-full"
      currentStep={currentPage + 1}
      totalSteps={lockedTransferStore.get().pipe(
        Option.map((lts) => lts.steps.length),
        Option.getOrElse(() =>
          transferSteps.pipe(
            Option.map((ts) => ts.length),
            Option.getOrElse(() => 1),
          ),
        ),
      )}
      stepDescriptions={lockedTransferStore.get().pipe(
        Option.map((lts) => lts.steps.map(TransferStep.description)),
        Option.orElse(() =>
          transferSteps.pipe(
            Option.map((ts) => ts.map(TransferStep.description)),
          ),
        ),
        Option.getOrElse(() => ["Configure your transfer"]),
      )}
    />
  </div>

  <div class="relative flex w-full grow">
    {#if currentStepTag === "Filling"}
      <div class="flex w-full grow" in:flyRight out:flyLeft>
        <FillingPage
          onContinue={handleActionButtonClick}
          {statusMessage}
          {transferErrors}
          onErrorClose={() => {
            transferErrors = Option.none();
          }}
          loading={isLoading}
        />
      </div>
    {/if}
    {#if currentStepTag === "CheckReceiver"}
      <div class="flex w-full grow" in:flyLeft out:flyLeft>
        <CheckReceiverPage
          stepIndex={currentPage + 1}
          onBack={goToPreviousPage}
          onSubmit={goToNextPage}
        />
      </div>
    {/if}
    {#if currentStepTag === "ApprovalRequired"}
      <div class="flex w-full grow" in:flyRight out:flyLeft>
        <ApprovalPage
          stepIndex={currentPage + 1}
          onBack={goToPreviousPage}
          onApprove={handleActionButtonClick}
          {actionButtonText}
        />
      </div>
    {/if}
    {#if currentStepTag === "SubmitInstruction"}
      <div class="flex w-full grow" in:flyLeft out:flyLeft>
        <SubmitPage
          stepIndex={currentPage + 1}
          onCancel={newTransfer}
          onSubmit={handleActionButtonClick}
          {actionButtonText}
        />
      </div>
    {/if}
    {#if currentStepTag === "WaitForIndex"}
      <IndexPage {newTransfer} />
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
    <pre>{JSON.stringify(lockedTransferStore.transfer, null, 2)}</pre>
  {/key}

  {#if Option.isSome(transferSteps)}
    <div class="mt-4">
      <strong>Steps:</strong>
      <pre>{JSON.stringify(transferSteps.value, null, 2)}</pre>
    </div>
  {/if}
{/if}
