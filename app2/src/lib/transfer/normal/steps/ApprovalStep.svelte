<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import InsetError from "$lib/components/model/InsetError.svelte"
import Label from "$lib/components/ui/Label.svelte"
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
import { cosmosSpenderAddresses } from "$lib/constants/spender-addresses.ts"
import { erc20Abi, http, isHex, toHex } from "viem"
import type { TransferStep } from "$lib/transfer/normal/steps"

type Props = {
  stepIndex: number
  step: TransferStep.ApprovalRequired
  onBack: () => void
  onApprove: () => void
  actionButtonText: string
}

const { step, onBack, onApprove, actionButtonText }: Props = $props()

let showError = $state(false)
let isSubmitting = $state(false)
let error = $state<Option.Option<unknown>>(Option.none())

let ets = $state<TransactionSubmissionEvm>(TransactionSubmissionEvm.Filling())
let cts = $state<TransactionSubmissionCosmos>(TransactionSubmissionCosmos.Filling())

const isButtonEnabled = $derived(
  !isSubmitting &&
    ((ets._tag === "Filling" && cts._tag === "Filling") ||
      evmHasFailedExit(ets) ||
      cosmosHasFailedExit(cts))
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
  isSubmitting = true
  error = Option.none()

  try {
    const chain = step.intent.sourceChain
    const rpcType = chain.rpc_type

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
                args: [step.intent.ucs03address, step.requiredAmount]
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
          const signingClient = yield* getCosmWasmClient(chain, cosmosStore.connectedWallet)
          const sender = yield* chain.getDisplayAddress(wallets.cosmosAddress.value)
          const spender = cosmosSpenderAddresses[chain.universal_chain_id]

          do {
            cts = yield* Effect.promise(() =>
              nextStateCosmos(cts, chain, signingClient, sender, step.token, {
                increase_allowance: {
                  spender,
                  amount: step.requiredAmount
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
  Effect.runPromise(submit).catch(err => {
    console.error("Uncaught approval error:", err)
    error = Option.some(err)
    isSubmitting = false
  })
}

const sourceChain = step.intent.sourceChain
const massagedDenom = isHex(step.token) ? step.token : toHex(step.token)
</script>

<div class="grow relative min-w-full p-4 flex flex-col justify-between h-full">
  <div class="grow flex flex-col gap-4">
    <h3 class="text-lg font-semibold">
      Approve
      <TokenComponent chain={sourceChain} denom={massagedDenom} />
    </h3>

    <section>
      <Label>Current</Label>
      <TokenComponent
        chain={sourceChain}
        denom={massagedDenom}
        amount={step.currentAllowance}
      />
    </section>

    <section>
      <Label>Required</Label>
      <TokenComponent
        chain={sourceChain}
        denom={massagedDenom}
        amount={step.requiredAmount}
      />
    </section>

    <p class="text-sm text-zinc-400">
      You need to approve Union to send
      <TokenComponent chain={sourceChain} denom={massagedDenom} />.
      This is a one-time approval for this token.
    </p>
  </div>

  <div class="flex justify-between mt-4">
    <Button variant="secondary" onclick={onBack} disabled={!isButtonEnabled}>
      Back
    </Button>
    {#if Option.isSome(error)}
      <div class="flex justify-end gap-2">
        <Button variant="danger" onclick={() => (showError = true)}>Error</Button>
        <Button variant="primary" onclick={handleSubmit} disabled={!isButtonEnabled}>
          {getSubmitButtonText}
        </Button>
      </div>
    {:else}
      <Button variant="primary" onclick={handleSubmit} disabled={!isButtonEnabled}>
        {getSubmitButtonText}
      </Button>
    {/if}
  </div>

  <InsetError
    open={showError}
    error={Option.isSome(error) ? error.value : null}
    onClose={() => {
      showError = false
      error = Option.none()
    }}
  />
</div>
