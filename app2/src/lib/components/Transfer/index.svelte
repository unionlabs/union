<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import StepProgressBar from "$lib/components/ui/StepProgressBar.svelte"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import FillingPage from "./pages/FillingPage.svelte"
import ApprovalPage from "./pages/ApprovalPage.svelte"
import SubmitPage from "./pages/SubmitPage.svelte"
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
import { fly } from "svelte/transition"
import type {TransferIntent} from "$lib/components/Transfer/state/filling/create-intents.ts"
import { generateMultisigTx } from "$lib/utils/multisig.ts"

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
  if (transfer.signingMode === "multi") {
    console.log("SIGNING MODE IS MULTISIG")
    const b = Effect.runSync(generateMultisigTx(localIntent))
    console.log({ b })
    return
  }
  if (Option.isNone(transferSteps)) return
  const currentStep = transferSteps.value[currentPage]

  if (TransferStep.is("Filling")(currentStep)) {
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
  transferHashStore.reset()
  wallets.clearInputAddress()
}

let localIntent = $state<TransferIntent>()

$effect(() => {
  if (currentPage !== 0) return
  interruptFiber()

  isLoading = true
  transferSteps = Option.none()
  transferErrors = Option.none()

  const machineEffect = Effect.gen(function* () {
    let currentState: CreateTransferState = CreateTransferState.Filling()
    let intent: TransferIntent

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

      if (Option.isSome(result.intent)) {
        intent = result.intent.value
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

    if (intent) {
      localIntent = intent

      if (Option.isSome(intent.allowances)) {
        const allowances = intent.allowances.value

        for (let i = 0; i < allowances.length; i++) {
          const allowance = allowances[i]
          const context = intent.contexts[i]

          steps.push(
            TransferStep.ApprovalRequired({
              token: allowance.token,
              requiredAmount: allowance.requiredAmount,
              currentAllowance: allowance.currentAllowance,
              context
            })
          )
        }
      }

      if (Option.isSome(intent.instruction)) {
        const instruction = intent.instruction.value

        for (const context of intent.contexts) {
          steps.push(
            TransferStep.SubmitInstruction({ instruction, context }),
            TransferStep.WaitForIndex({ context })
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

const flyLeft = (node: Element) =>
  fly(node, {
    x: -300,
    duration: 300,
    delay: 0
  })

const flyRight = (node: Element) =>
  fly(node, {
    x: 300,
    duration: 300,
    delay: 0
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
      totalSteps={transferSteps.pipe(
        Option.map(ts => ts.length),
        Option.getOrElse(() => 1)
      )}
      stepDescriptions={transferSteps.pipe(
        Option.map(ts => ts.map(TransferStep.description)),
        Option.getOrElse(() => ["Configure your transfer"])
      )}
    />
  </div>

  <div class="grid w-full grow">
    {#if currentPage === 0}
      <div class="flex grow col-start-1 col-end-2 row-start-1 row-end-2" in:flyRight out:flyLeft>
        <FillingPage
          onContinue={handleActionButtonClick}
          {statusMessage}
          {transferErrors}
          onErrorClose={() => {
          transferErrors = Option.none()
        }}
          loading={isLoading}
        />
      </div>
    {:else if Option.isSome(transferSteps)}
      {#if Option.isSome(Arr.get(currentPage)(transferSteps.value))}
        {@const step = Arr.get(currentPage)(transferSteps.value).value}

        {#if TransferStep.is("CheckReceiver")(step)}
          <div class="flex grow col-start-1 col-end-2 row-start-1 row-end-2" in:flyLeft out:flyLeft>
            <CheckReceiverPage
              stepIndex={currentPage + 1}
              step={step}
              onBack={goToPreviousPage}
              onSubmit={goToNextPage}
            />
          </div>
        {:else if TransferStep.is("ApprovalRequired")(step)}
          <div class="flex grow col-start-1 col-end-2 row-start-1 row-end-2" in:flyRight out:flyLeft>
            <ApprovalPage
              stepIndex={currentPage + 1}
              step={step}
              onBack={goToPreviousPage}
              onApprove={handleActionButtonClick}
              {actionButtonText}
            />
          </div>
        {:else if TransferStep.is("SubmitInstruction")(step)}
          <div class="flex grow col-start-1 col-end-2 row-start-1 row-end-2" in:flyLeft out:flyLeft>
            <SubmitPage
              stepIndex={currentPage + 1}
              step={step}
              onCancel={newTransfer}
              onSubmit={handleActionButtonClick}
              {actionButtonText}
            />
          </div>
        {:else if TransferStep.is("WaitForIndex")(step)}
          <div class="flex grow col-start-1 col-end-2 row-start-1 row-end-2" in:flyLeft out:flyLeft>
            <IndexPage {newTransfer} step={step} />
          </div>
        {/if}
      {/if}
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

