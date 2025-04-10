<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import StepProgressBar from "$lib/components/ui/StepProgressBar.svelte"
import { LockedTransfer } from "./locked-transfer.ts"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import FillingPage from "./pages/FillingPage.svelte"
import ApprovalPage from "./pages/ApprovalPage.svelte"
import SubmitPage from "./pages/SubmitPage.svelte"
import { lockedTransferStore } from "./locked-transfer.svelte.ts"
import { Effect, Option } from "effect"
import * as TransferStep from "./transfer-step.ts"
import IndexPage from "$lib/components/Transfer/pages/IndexPage.svelte"
import {
  CreateTransferState,
  createTransferState,
  type StateResult
} from "$lib/components/Transfer/state/filling/index.ts"
import type { TransferFlowError } from "$lib/components/Transfer/state/errors.ts"

let currentPage = $state(0)
let isLoading = $state(false)
let transferSteps = $state<Option.Option<Array<TransferStep.TransferStep>>>(Option.none())
let transferError = $state<Option.Option<TransferFlowError>>(Option.none())
let statusMessage = $state("")

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
  if (currentPage < 0 || currentPage >= steps.length || !steps[currentPage]) {
    return "Submit"
  }
  const currentStep = steps[currentPage]
  if (currentPage === steps.length - 1) return "Complete"
  return TransferStep.match(currentStep, {
    Filling: () => "Continue",
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
  if (TransferStep.is("ApprovalRequired")(currentStep)) goToNextPage()
  if (TransferStep.is("SubmitInstruction")(currentStep)) goToNextPage()
}

$effect(() => {
  isLoading = true
  transferSteps = Option.none()
  transferError = Option.none()
  statusMessage = "Starting transfer process..."

  const frozenTransfer = {
    ...transfer,
    sourceChain: transfer.sourceChain,
    destinationChain: transfer.destinationChain,
    baseToken: transfer.baseToken,
    channel: transfer.channel,
    parsedAmount: transfer.parsedAmount,
    intents: transfer.intents,
    derivedSender: transfer.derivedSender,
    ucs03address: transfer.ucs03address
  }

  const machineEffect = Effect.gen(function* () {
    let currentState: CreateTransferState = CreateTransferState.Filling()
    let finalOrders: Array<unknown> = []
    let finalAllowances: Array<{
      token: string
      requiredAmount: string
      currentAllowance: string
    }> = []

    while (true) {
      const result: StateResult = yield* createTransferState(currentState, frozenTransfer)

      statusMessage = result.message

      if (Option.isSome(result.error)) {
        transferError = result.error
        transferSteps = Option.none()
        isLoading = false
        return
      }

      if (Option.isSome(result.nextState)) {
        currentState = result.nextState.value
        continue
      }

      if (Option.isSome(result.orders)) {
        finalOrders = result.orders.value
      }

      if (Option.isSome(result.allowances)) {
        finalAllowances = result.allowances.value
      }
      break
    }

    const steps: Array<TransferStep.TransferStep> = [TransferStep.Filling()]

    steps.push(
      ...finalAllowances
        .filter(
          ({ requiredAmount, currentAllowance }) =>
            BigInt(currentAllowance) < BigInt(requiredAmount)
        )
        .map(({ token, requiredAmount, currentAllowance }) =>
          TransferStep.ApprovalRequired({
            token,
            requiredAmount: BigInt(requiredAmount),
            currentAllowance: BigInt(currentAllowance)
          })
        )
    )

    if (finalOrders.length > 0) {
      steps.push(TransferStep.SubmitInstruction({ instruction: finalOrders[0] }))
      steps.push(TransferStep.WaitForIndex())
    }

    transferSteps = Option.some(steps)
    isLoading = false
  })

  Effect.runFork(machineEffect)
})
</script>

<Card
        divided
        class="w-sm my-24 relative self-center flex flex-col justify-between min-h-[450px] overflow-hidden"
>
  <div class="w-full">
    <StepProgressBar
            class="w-full"
            currentStep={currentPage + 1}
            totalSteps={lockedTransferStore.get().pipe(
        Option.map((lts) => lts.steps.length),
        Option.getOrElse(() =>
          transferSteps.pipe(Option.map((ts) => ts.length), Option.getOrElse(() => 1))
        )
      )}
            stepDescriptions={lockedTransferStore.get().pipe(
        Option.map((lts) => lts.steps.map(TransferStep.description)),
        Option.orElse(() => transferSteps.pipe(Option.map((ts) => ts.map(TransferStep.description)))),
        Option.getOrElse(() => ["Configure your transfer"])
      )}
    />
  </div>

  <div class="relative flex-1 overflow-hidden">
    <div
            class="absolute inset-0 flex transition-transform duration-300 ease-in-out"
            style="transform: translateX(-{currentPage * 100}%);"
    >
      <FillingPage
              onContinue={handleActionButtonClick}
              {actionButtonText}
              gotSteps={Option.isSome(transferSteps) && transferSteps.value.length > 1}
              loading={isLoading}
      />

      {#if Option.isSome(lockedTransferStore.get())}
        {#each lockedTransferStore.get().value.steps.slice(1) as step, i}
          {#if TransferStep.is("ApprovalRequired")(step)}
            <ApprovalPage
                    stepIndex={i + 1}
                    onBack={goToPreviousPage}
                    onApprove={handleActionButtonClick}
                    {actionButtonText}
            />
          {:else if TransferStep.is("SubmitInstruction")(step)}
            <SubmitPage
                    stepIndex={i + 1}
                    onBack={goToPreviousPage}
                    onSubmit={handleActionButtonClick}
                    {actionButtonText}
            />
          {:else if TransferStep.is("WaitForIndex")(step)}
            <IndexPage newTransfer={() => {}} />
          {/if}
        {/each}
      {/if}
    </div>
  </div>
</Card>

{#key statusMessage}
  <p>{statusMessage}</p>
  <pre>{JSON.stringify(lockedTransferStore.transfer, null, 2)}</pre>
{/key}

{#if Option.isSome(transferError)}
  <strong>Error</strong>
  <pre>{JSON.stringify(transferError.value, null, 2)}</pre>
{/if}

{#if Option.isSome(transferSteps)}
  <div class="mt-4">
    <strong>Steps:</strong>
    <pre>{JSON.stringify(transferSteps.value, null, 2)}</pre>
  </div>
{/if}


