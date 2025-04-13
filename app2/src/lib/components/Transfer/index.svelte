<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import StepProgressBar from "$lib/components/ui/StepProgressBar.svelte"
import { LockedTransfer } from "./locked-transfer.ts"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import FillingPage from "./pages/FillingPage.svelte"
import ApprovalPage from "./pages/ApprovalPage.svelte"
import SubmitPage from "./pages/SubmitPage.svelte"
import { lockedTransferStore } from "./locked-transfer.svelte.ts"
import { Effect, Option, Fiber, Match } from "effect"
import * as TransferStep from "./transfer-step.ts"
import IndexPage from "$lib/components/Transfer/pages/IndexPage.svelte"
import {
  CreateTransferState,
  createTransferState,
  type StateResult
} from "$lib/components/Transfer/state/filling/index.ts"
import type { TransferFlowError } from "$lib/components/Transfer/state/errors.ts"
import type { Batch } from "@unionlabs/sdk/ucs03/instruction.ts"
import { transferHashStore } from "$lib/stores/transfer-hash.svelte.ts"
import { constVoid, flow, identity, pipe } from "effect/Function"
import CheckReceiverPage from "./pages/CheckReceiverPage.svelte"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import { beforeNavigate } from "$app/navigation"
import { onMount } from "svelte"

let currentPage = $state(0)
let isLoading = $state(false)
let transferSteps = $state<Option.Option<Array<TransferStep.TransferStep>>>(Option.none())
let transferError = $state<Option.Option<TransferFlowError>>(Option.none())
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
  transferError = Option.none()
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
    derivedReceiver: transfer.derivedReceiver,
    ucs03address: transfer.ucs03address
  }

  const machineEffect = Effect.gen(function* () {
    let currentState: CreateTransferState = CreateTransferState.Filling()
    let finalOrders: Array<Batch> = []
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
        currentFiber = Option.none()
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

    const isReceiverInWallet = pipe(
      Option.all({
        destinationChain: frozenTransfer.destinationChain,
        receiver: frozenTransfer.derivedReceiver
      }),
      Option.flatMap(({ destinationChain, receiver }) => {
        const walletaddr = wallets.getAddressForChain(destinationChain)

        console.log({ walletaddr, receiver })

        return Option.map(walletaddr, x => x.toLowerCase() === receiver.toLowerCase())
      }),
      Option.getOrElse(() => false)
    )

    if (!isReceiverInWallet) {
      console.log({ frozenTransfer })
      steps.push(
        TransferStep.CheckReceiver({
          receiver: frozenTransfer.derivedReceiver,
          destinationChain: frozenTransfer.destinationChain
        })
      )
    }

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
    currentFiber = Option.none()
  })

  const fiber = Effect.runFork(machineEffect as Effect.Effect<void, never, never>)
  currentFiber = Option.some(fiber)
})

const fillingError = $derived(
  pipe(
    transferError,
    Option.flatMap(
      flow(
        Match.value,
        Match.tags({
          OrderCreationError: identity
        }),
        Match.orElse(() => null),
        Option.fromNullable
      )
    )
  )
)

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
</script>

<Card
        divided
        class="w-sm mt-24 relative self-center flex flex-col justify-between min-h-[450px] overflow-hidden"
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

  <div class="relative flex-1 overflow-hidden">
    <div
            class="absolute inset-0 flex transition-transform duration-300 ease-in-out"
            style="transform: translateX(-{currentPage * 100}%);"
    >
      <FillingPage
              onContinue={handleActionButtonClick}
              {actionButtonText}
              topError={fillingError}
              onErrorClose={() => {
          transferError = Option.none();
        }}
              gotSteps={Option.isSome(transferSteps) &&
          transferSteps.value.length > 1}
              loading={isLoading}
      />

      {#if Option.isSome(lockedTransferStore.get())}
        {#each lockedTransferStore.get().value.steps.slice(1) as step, i}
          {#if TransferStep.is("CheckReceiver")(step)}
            <CheckReceiverPage
                    stepIndex={i + 1}
                    onBack={goToPreviousPage}
                    onSubmit={goToNextPage}
            />
          {:else if TransferStep.is("ApprovalRequired")(step)}
            <ApprovalPage
                    stepIndex={i + 1}
                    onBack={goToPreviousPage}
                    onApprove={handleActionButtonClick}
                    {actionButtonText}
            />
          {:else if TransferStep.is("SubmitInstruction")(step)}
            <SubmitPage
                    stepIndex={i + 1}
                    onCancel={newTransfer}
                    onSubmit={handleActionButtonClick}
                    {actionButtonText}
            />
          {:else if TransferStep.is("WaitForIndex")(step)}
            <IndexPage {newTransfer}/>
          {/if}
        {/each}
      {/if}
    </div>
  </div>
</Card>

{#if showDetails}
  {#if Option.isSome(transferError)}
    <strong>Error</strong>
    <pre class="text-wrap">{JSON.stringify(transferError.value, null, 2)}</pre>
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

