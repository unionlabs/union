<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import TokenComponent from "$lib/components/model/TokenComponent.svelte";
  import { Array as A, Effect, Option, Struct } from "effect";
  import { lockedTransferStore } from "../locked-transfer.svelte.ts";
  import { createViemPublicClient } from "@unionlabs/sdk/evm";
  import * as TransferStep from "../transfer-step.ts";
  import { erc20Abi, http } from "viem";
  import {
    nextStateEvm,
    hasFailedExit,
    isComplete,
    TransactionSubmissionEvm,
  } from "$lib/components/Transfer/state/evm.ts";
  import { getWalletClient } from "$lib/services/evm/clients.ts";
  import { setPassword } from "@effect/platform/Url";
  import type { LockedTransfer } from "../locked-transfer.ts";

  type Props = {
    stepIndex: number;
    onBack: () => void;
    onApprove: () => void;
    actionButtonText: string;
  };

  const { stepIndex, onBack, onApprove, actionButtonText }: Props = $props();

  const lts = lockedTransferStore.get();

  // Get the step data from the locked transfer store
  const step: Option.Option<typeof TransferStep.TransferStep> = $derived(
    lts.pipe(
      Option.map(Struct.get("steps")),
      Option.flatMap(
        Option.liftPredicate(
          (steps) => stepIndex < 0 || stepIndex >= steps.length,
        ),
      ),
      Option.flatMap(A.get(stepIndex)),
      Option.flatMap(Option.liftPredicate(TransferStep.is("ApprovalRequired"))),
    ),
  );

  const sourceChain = $derived(
    lts.pipe(Option.map((ltss) => ltss.sourceChain)),
  );

  let ts = $state<TransactionSubmissionEvm>(TransactionSubmissionEvm.Filling());

  const submit = Effect.gen(function* () {
    if (Option.isNone(step) || Option.isNone(lts)) return;

    const viemChain = lts.value.sourceChain.toViemChain();
    if (Option.isNone(viemChain)) return;

    const publicClient = yield* createViemPublicClient({
      chain: viemChain.value,
      transport: http(),
    });

    const walletClient = yield* getWalletClient(lts.value.sourceChain);

    do {
      ts = yield* Effect.tryPromise({
        try: () =>
          nextStateEvm(ts, viemChain.value, publicClient, walletClient, {
            chain: viemChain.value,
            account: walletClient.account,
            address: step.value.token,
            abi: erc20Abi,
            functionName: "approve",
            args: [lts.value.channel.source_port_id, step.value.requiredAmount],
          }),
        catch: (error) =>
          error instanceof Error ? error : new Error("Unknown error"),
      });

      if (isComplete(ts)) {
        onApprove();
        break;
      }
    } while (!hasFailedExit(ts));

    return ts;
  });
</script>

<div class="min-w-full p-4 flex flex-col justify-between h-full">
  {#if Option.isSome(step) && Option.isSome(sourceChain)}
    <div class="flex-1">
      <h3 class="text-lg font-semibold mb-4">Approve Token</h3>
      <div class="bg-zinc-800 rounded-lg p-4 mb-4">
        <div class="mb-2">
          <span class="text-zinc-400">Token:</span>
          <span class="font-mono text-sm ml-2">
            <TokenComponent
              chain={sourceChain.value}
              denom={step.value.token}
            />
          </span>
        </div>
        <div class="mb-2">
          <span class="text-zinc-400">Current Allowance:</span>
          <span class="font-mono text-sm ml-2"
            >{step.value.currentAllowance.toString()}</span
          >
        </div>
        <div>
          <span class="text-zinc-400">Required Amount:</span>
          <span class="font-mono text-sm ml-2"
            >{step.value.requiredAmount.toString()}</span
          >
        </div>
      </div>
      <p class="text-sm text-zinc-400">
        You need to approve the smart contract to spend your tokens. This is a
        one-time approval for this token.
      </p>
    </div>

    <div class="flex justify-between mt-4">
      <Button variant="secondary" onclick={onBack}>Back</Button>
      <Button variant="primary" onclick={() => Effect.runPromise(submit)}>
        {actionButtonText}
      </Button>
    </div>
  {:else}
    <div class="flex items-center justify-center h-full">
      <p class="text-zinc-400">Loading approval details...</p>
    </div>
  {/if}
</div>
