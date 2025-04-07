<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte"
  import TokenComponent from "$lib/components/model/TokenComponent.svelte"
  import { Effect, Match, Option, Array, Struct } from "effect"
  import { lockedTransferStore } from "../locked-transfer.svelte.ts"
  import { ApprovalRequired } from "../transfer-step.ts"
  import { createViemPublicClient } from "@unionlabs/sdk/evm"
  import {erc20Abi, fromHex, http, toHex} from "viem"
  import {
    hasFailedExit as evmHasFailedExit,
    isComplete as evmIsComplete,
    nextStateEvm,
    TransactionSubmissionEvm
  } from "$lib/components/Transfer/state/evm.ts"
  import {
    nextStateCosmos,
    isComplete as cosmosIsComplete,
    hasFailedExit as cosmosHasFailedExit,
    TransactionSubmissionCosmos
  } from "$lib/components/Transfer/state/cosmos.ts"
  import { getWalletClient } from "$lib/services/evm/clients.ts"
  import Label from "$lib/components/ui/Label.svelte"
  import { is } from "../transfer-step.ts"
  import {getCosmWasmClient} from "$lib/services/cosmos/clients.ts";
  import {cosmosStore} from "$lib/wallet/cosmos";
  import {wallets} from "$lib/stores/wallets.svelte.ts";

  type Props = {
    stepIndex: number
    onBack: () => void
    onApprove: () => void
    actionButtonText: string
  }

  const {stepIndex, onBack, onApprove, actionButtonText}: Props = $props()

  const lts = lockedTransferStore.get()

  const step = $derived(
    lts.pipe(
      Option.map(Struct.get("steps")),
      Option.flatMap(Array.get(stepIndex)),
      Option.filter(is("ApprovalRequired"))
    )
  )

  const sourceChain = $derived(lts.pipe(Option.map(Struct.get("sourceChain"))))

  let ets = $state<TransactionSubmissionEvm>(TransactionSubmissionEvm.Filling())
  let cts = $state<TransactionSubmissionCosmos>(TransactionSubmissionCosmos.Filling())

  const isButtonEnabled = $derived(
    (ets._tag === "Filling" && cts._tag === "Filling") ||
    evmHasFailedExit(ets) ||
    cosmosHasFailedExit(cts)
  )

  const getSubmitButtonText = $derived(
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
    if (Option.isNone(step) || Option.isNone(lts)) return

    const sourceChainRpcType = lts.value.sourceChain.rpc_type

    yield* Match.value(sourceChainRpcType).pipe(
      Match.when("evm", () =>
        Effect.gen(function* () {
          // Use the component-level state variable
          const viemChain = lts.value.sourceChain.toViemChain()
          if (Option.isNone(viemChain)) return Effect.succeed(null)

          const publicClient = yield* createViemPublicClient({
            chain: viemChain.value,
            transport: http()
          })

          const walletClient = yield* getWalletClient(lts.value.sourceChain)

          do {
            ets = yield* Effect.tryPromise({
              try: () =>
                nextStateEvm(ets, viemChain.value, publicClient, walletClient, {
                  chain: viemChain.value,
                  account: walletClient.account,
                  address: step.value.token,
                  abi: erc20Abi,
                  functionName: "approve",
                  args: [lts.value.channel.source_port_id, step.value.requiredAmount]
                }),
              catch: error => (error instanceof Error ? error : new Error("Unknown error"))
            })

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
          const signingClient = yield* getCosmWasmClient(
            lts.value.sourceChain,
            cosmosStore.connectedWallet
          )

          const sender = yield* lts.value.sourceChain.getDisplayAddress(wallets.cosmosAddress.value)

          do {
            cts = yield* Effect.tryPromise({
              try: () =>
                nextStateCosmos(
                  cts,
                  lts.value.sourceChain,
                  signingClient,
                  sender,
                  step.value.token,
                  {
                    increase_allowance: {
                      spender: "bbn1dy20pwy30hfqyxdzrmp33h47h4xdxht6phqecfp2jdnes6su9pysqq2kpw",
                      amount: step.value.requiredAmount,
                    }
                  }
                ),
              catch: error => (error instanceof Error ? error : new Error("Unknown error"))
            })

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
          yield* Effect.log("unknown chain type")
          return Effect.succeed("unknown chain type")
        })
      )
    )
  })
</script>

<div class="min-w-full p-4 flex flex-col justify-between h-full">
  {#if Option.isSome(step) && Option.isSome(sourceChain)}
    <div class="flex-1 flex flex-col gap-4">
      <h3 class="text-lg font-semibold">Approve
        <TokenComponent chain={sourceChain.value} denom={toHex(step.value.token)}/>
      </h3>
      <section>
        <Label>Current</Label>
        <TokenComponent chain={sourceChain.value} denom={toHex(step.value.token)} amount={step.value.currentAllowance}/>
      </section>
      <section>
        <Label>Required</Label>
        <TokenComponent chain={sourceChain.value} denom={toHex(step.value.token)} amount={step.value.requiredAmount}/>
      </section>
      <p class="text-sm text-zinc-400">
        You need to approve Union to send
        <TokenComponent chain={sourceChain.value} denom={toHex(step.value.token)}/>
        .
        This is a one-time approval for this token.
      </p>
    </div>

    <div class="flex justify-between mt-4">
      <Button
              variant="secondary"
              onclick={onBack}
              disabled={!isButtonEnabled}
      >
        Back
      </Button>
      <Button
              variant="primary"
              onclick={() => Effect.runPromise(submit)}
              disabled={!isButtonEnabled}
      >
        {getSubmitButtonText}
      </Button>
    </div>
  {:else}
    <div class="flex items-center justify-center h-full">
      <p class="text-zinc-400">Loading approval details...</p>
    </div>
  {/if}
</div>

