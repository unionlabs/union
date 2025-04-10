<script lang="ts">
  import Card from "$lib/components/ui/Card.svelte";
  import StepProgressBar from "$lib/components/ui/StepProgressBar.svelte";
  import { LockedTransfer } from "./locked-transfer.ts";
  import { transfer } from "$lib/components/Transfer/transfer.svelte.ts";
  import FillingPage from "./pages/FillingPage.svelte";
  import ApprovalPage from "./pages/ApprovalPage.svelte";
  import SubmitPage from "./pages/SubmitPage.svelte";
  import { lockedTransferStore } from "./locked-transfer.svelte.ts";
  import { Effect, Exit, Option } from "effect";
  import * as TransferStep from "./transfer-step.ts";
  import IndexPage from "$lib/components/Transfer/pages/IndexPage.svelte";
  import {
    CreateTransferState,
    createTransferState,
    type StateResult,
  } from "$lib/components/Transfer/state/filling/index.ts";

  let currentPage = $state(0);
  let loading = $state(false);
  let transferSteps = $state<Option.Option<Array<TransferStep.TransferStep>>>(
    Option.none(),
  );

  function goToNextPage() {
    if (
      Option.isSome(transferSteps) &&
      currentPage < transferSteps.value.length - 1
    ) {
      currentPage++;
    }
  }

  function goToPreviousPage() {
    if (currentPage > 0) {
      currentPage--;
      if (currentPage === 0) {
        lockedTransferStore.unlock();
      }
    }
  }

  let actionButtonText = $derived.by(() => {
    if (Option.isNone(transferSteps)) return "Submit";
    const steps = transferSteps.value;
    if (currentPage < 0 || currentPage >= steps.length || !steps[currentPage]) {
      return "Submit";
    }
    const currentStep = steps[currentPage];
    if (currentPage === steps.length - 1) {
      return "Complete";
    }
    return TransferStep.match(currentStep, {
      Filling: () => "Continue",
      ApprovalRequired: () => "Approve",
      SubmitInstruction: () => "Submit",
      WaitForIndex: () => "Submit",
    });
  });

  function handleActionButtonClick() {
    if (Option.isNone(transferSteps)) return;
    const currentStep = transferSteps.value[currentPage];
    if (TransferStep.is("Filling")(currentStep)) {
      if (Option.isNone(lockedTransferStore.get())) {
        const newLockedTransfer = LockedTransfer.fromTransfer(
          transfer.sourceChain,
          transfer.destinationChain,
          transfer.channel,
          transfer.parsedAmount,
          transfer.baseToken,
          transferSteps,
        );
        if (Option.isNone(newLockedTransfer)) {
          console.error("Failed to lock transfer values");
          return;
        }
        lockedTransferStore.lock(newLockedTransfer.value);
      }
      goToNextPage();
      return;
    }
    if (TransferStep.is("ApprovalRequired")(currentStep)) {
      goToNextPage();
      return;
    }
    if (TransferStep.is("SubmitInstruction")(currentStep)) {
      goToNextPage();
      return;
    }
  }

  let isLoading: boolean = $state(false);
  let statusMessage = $state("");

  // New effect block that runs the state machine and computes transferSteps inline.
  $effect(() => {
    isLoading = true;
    statusMessage = "Starting transfer process...";

    const runStateMachine = async () => {
      let currentState: CreateTransferState = CreateTransferState.Filling();
      let running = true;
      let finalOrders: Array<unknown> | undefined;
      let finalAllowances:
        | Array<{
            token: string;
            requiredAmount: string;
            currentAllowance: string;
          }>
        | undefined;

      while (running) {
        const exit = await Effect.runPromiseExit(
          createTransferState(currentState, transfer),
        );
        if (Exit.isSuccess(exit)) {
          const result: StateResult = exit.value;
          if (result.message) {
            statusMessage = result.message;
          }
          if (result.orders) {
            finalOrders = result.orders;
          }
          if (result.allowances) {
            finalAllowances = result.allowances;
          }
          if (result.nextState) {
            currentState = result.nextState;
          } else {
            running = false;
          }
        } else {
          statusMessage = "An error occurred";
          running = false;
        }
      }

      // Build transferSteps inline.
      const steps: Array<TransferStep.TransferStep> = [];
      steps.push(TransferStep.Filling());

      // Compute approval steps using the simplified allowance data.
      if (finalAllowances) {
        const approvalSteps = finalAllowances
          .filter(
            ({ requiredAmount, currentAllowance }) =>
              BigInt(currentAllowance) < BigInt(requiredAmount),
          )
          .map(({ token, requiredAmount, currentAllowance }) =>
            TransferStep.ApprovalRequired({
              token,
              requiredAmount: BigInt(requiredAmount),
              currentAllowance: BigInt(currentAllowance),
            }),
          );
        steps.push(...approvalSteps);
      }

      if (finalOrders) {
        steps.push(
          TransferStep.SubmitInstruction({ instruction: finalOrders }),
        );
        steps.push(TransferStep.WaitForIndex());
      }

      transferSteps = Option.some(steps);
      isLoading = false;
    };

    runStateMachine();
  });
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

  <!-- Sliding pages container -->
  <div class="relative flex-1 overflow-hidden">
    <!-- Pages wrapper with horizontal sliding -->

    <div
      class="absolute inset-0 flex transition-transform duration-300 ease-in-out"
      style="transform: translateX(-{currentPage * 100}%);"
    >
      <!-- Page 1: Filling -->
      <FillingPage
        onContinue={handleActionButtonClick}
        {actionButtonText}
        gotSteps={Option.isSome(transferSteps) &&
          transferSteps.value.length > 1}
        {loading}
      />

      <!-- Dynamic pages for each step -->
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
            <IndexPage stepIndex={i + 1} />
          {/if}
        {/each}
      {/if}
    </div>
  </div>
</Card>
{#key statusMessage}
  <p>{statusMessage}</p>
  <pre>

    {JSON.stringify(lockedTransferStore.transfer, null, 2)}
  </pre>
{/key}
