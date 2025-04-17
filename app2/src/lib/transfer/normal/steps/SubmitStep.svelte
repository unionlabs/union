<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import { Cause, Effect, Exit, Match, Option } from "effect"
import {
  hasFailedExit as evmHasFailedExit,
  isComplete as evmIsComplete,
  nextStateEvm,
  TransactionSubmissionEvm
} from "$lib/transfer/shared/services/write-evm.ts"
import {
  hasFailedExit as cosmosHasFailedExit,
  isComplete as cosmosIsComplete,
  nextStateCosmos,
  TransactionSubmissionCosmos
} from "$lib/transfer/shared/services/write-cosmos.ts"
import { generateSalt } from "@unionlabs/sdk/utils"
import { getConnectorClient, http, type GetConnectorClientErrorType } from "@wagmi/core"
import { createViemPublicClient, createViemWalletClient } from "@unionlabs/sdk/evm"
import { custom, encodeAbiParameters, fromHex } from "viem"
import { wagmiConfig } from "$lib/wallet/evm/wagmi-config.ts"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import { getCosmWasmClient } from "$lib/services/cosmos/clients.ts"
import { cosmosStore } from "$lib/wallet/cosmos"
import { ucs03ZkgmAbi } from "$lib/abi/ucs03.ts"
import { instructionAbi } from "@unionlabs/sdk/evm/abi"
import { encodeAbi } from "@unionlabs/sdk/ucs03/instruction.ts"
import { getTimeoutInNanoseconds24HoursFromNow } from "@unionlabs/sdk/utils/timeout.ts"
import { transferHashStore } from "$lib/stores/transfer-hash.svelte.ts"
import { ConnectorClientError } from "$lib/services/transfer"
import { isValidBech32ContractAddress } from "$lib/utils"
import Label from "$lib/components/ui/Label.svelte"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import InsetError from "$lib/components/model/InsetError.svelte"
import type { SubmitInstruction } from "../steps.ts"

type Props = {
  stepIndex: number
  step: SubmitInstruction
  onSubmit: () => void
  onCancel?: () => void
  actionButtonText: string
}

const { stepIndex, step, onSubmit, onCancel, actionButtonText }: Props = $props()

let showError = $state(false)

let ets = $state<TransactionSubmissionEvm>(TransactionSubmissionEvm.Filling())
let cts = $state<TransactionSubmissionCosmos>(TransactionSubmissionCosmos.Filling())
let error = $state<Option.Option<unknown>>(Option.none())
let isSubmitting = $state(false)

const needsRetry = $derived(evmHasFailedExit(ets) || cosmosHasFailedExit(cts))

const isButtonEnabled = $derived(
  !isSubmitting && ((ets._tag === "Filling" && cts._tag === "Filling") || needsRetry)
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
            : needsRetry
              ? "Try Again"
              : actionButtonText
)

const resetState = () => {
  ets = TransactionSubmissionEvm.Filling()
  cts = TransactionSubmissionCosmos.Filling()
  error = Option.none()
  isSubmitting = false
}

const submit = Effect.gen(function* () {
  if (needsRetry) {
    resetState()
    return
  }

  isSubmitting = true
  error = Option.none()

  const context = step.context
  const instruction = step.instruction

  yield* Match.value(context.sourceChain.rpc_type).pipe(
    Match.when("evm", () =>
      Effect.gen(function* () {
        const viemChain = context.sourceChain.toViemChain()
        if (Option.isNone(viemChain)) return Effect.succeed(null)

        const publicClient = yield* createViemPublicClient({
          chain: viemChain.value,
          transport: http()
        })

        const connectorClient = yield* Effect.tryPromise({
          try: () => getConnectorClient(wagmiConfig),
          catch: err => new ConnectorClientError({ cause: err as GetConnectorClientErrorType })
        })

        const walletClient = yield* createViemWalletClient({
          account: connectorClient.account,
          chain: viemChain.value,
          transport: custom(connectorClient)
        })

        const timeout = getTimeoutInNanoseconds24HoursFromNow()
        const salt = yield* generateSalt("evm")

        do {
          ets = yield* Effect.promise(() =>
            nextStateEvm(ets, viemChain.value, publicClient, walletClient, {
              chain: viemChain.value,
              account: connectorClient.account,
              address: context.channel.source_port_id,
              abi: ucs03ZkgmAbi,
              functionName: "send",
              args: [
                context.channel.source_channel_id,
                0n,
                timeout,
                salt,
                {
                  opcode: instruction.opcode,
                  version: instruction.version,
                  operand: encodeAbi(instruction)
                }
              ]
            })
          )

          if (ets._tag === "SwitchChainComplete" || ets._tag === "WriteContractComplete") {
            yield* Exit.matchEffect(ets.exit, {
              onFailure: cause =>
                Effect.sync(() => {
                  error = Option.some(Cause.squash(cause))
                }),
              onSuccess: () => Effect.sync(() => (error = Option.none()))
            })
          }

          const result = evmIsComplete(ets)
          if (result) {
            transferHashStore.startPolling(result)
            onSubmit()
            break
          }
        } while (!evmHasFailedExit(ets))

        return Effect.succeed(ets)
      })
    ),

    Match.when("cosmos", () =>
      Effect.gen(function* () {
        const signingClient = yield* getCosmWasmClient(
          context.sourceChain,
          cosmosStore.connectedWallet
        )
        const sender = yield* context.sourceChain.getDisplayAddress(wallets.cosmosAddress.value)
        const isNative = !isValidBech32ContractAddress(fromHex(context.baseToken, "string"))
        const timeout = getTimeoutInNanoseconds24HoursFromNow().toString()
        const salt = yield* generateSalt("cosmos")

        do {
          cts = yield* Effect.promise(() =>
            nextStateCosmos(
              cts,
              context.sourceChain,
              signingClient,
              sender,
              fromHex(context.channel.source_port_id, "string"),
              {
                send: {
                  channel_id: context.channel.source_channel_id,
                  timeout_height: "0",
                  timeout_timestamp: timeout,
                  salt,
                  instruction: encodeAbiParameters(instructionAbi, [
                    instruction.version,
                    instruction.opcode,
                    encodeAbi(instruction)
                  ])
                }
              },
              isNative
                ? [
                    {
                      denom: fromHex(context.baseToken, "string"),
                      amount: context.baseAmount.toString()
                    }
                  ]
                : undefined
            )
          )

          if (cts._tag === "SwitchChainComplete" || cts._tag === "WriteContractComplete") {
            yield* Exit.matchEffect(cts.exit, {
              onFailure: cause =>
                Effect.sync(() => {
                  error = Option.some(Cause.squash(cause))
                }),
              onSuccess: () => Effect.sync(() => (error = Option.none()))
            })
          }

          const result = cosmosIsComplete(cts)
          if (result) {
            transferHashStore.startPolling(`0x${result}`)
            onSubmit()
            break
          }
        } while (!cosmosHasFailedExit(cts))

        return Effect.succeed(cts)
      })
    ),

    Match.orElse(() =>
      Effect.gen(function* () {
        yield* Effect.log("Unknown chain type")
        error = Option.some({ _tag: "UnknownError", cause: "Unsupported chain type" })
        return Effect.succeed("unsupported")
      })
    )
  )

  isSubmitting = false
})

const handleSubmit = () => {
  error = Option.none()
  showError = false
  if (needsRetry) {
    resetState()
    return
  }

  Effect.runPromise(submit).catch(err => {
    console.error("Uncaught error in transfer process:", err)
    error = Option.some({
      _tag: err.name || "UnhandledError",
      cause: err.message || "An unexpected error occurred"
    })
    isSubmitting = false
  })
}
</script>


<div class="relative min-w-full p-4 flex flex-col justify-between h-full">
  {#if step && step.context.sourceChain && step.context.destinationChain}
    <div class="flex-1 flex flex-col gap-4">
      <h3 class="text-lg font-semibold">Submit Transfer</h3>
      <section>
        <Label>From</Label>
        <ChainComponent chain={step.context.sourceChain} />
      </section>

      <section>
        <Label>To</Label>
        <ChainComponent chain={ step.context.destinationChain} />
      </section>
      <p class="text-sm text-zinc-400">
        This will initiate the transfer on
        <ChainComponent chain={step.context.sourceChain} />. You'll need to confirm the
        transfer in your wallet.
      </p>
    </div>

    <div class="flex justify-between mt-4">
      <Button
        variant="secondary"
        onclick={onCancel}
        disabled={!isButtonEnabled}
      >
        Cancel
      </Button>
      {#if Option.isSome(error)}
        <div class="flex gap-2">
          <Button variant="danger" onclick={() => (showError = true)}>
            Error
          </Button>
          <Button
            variant="primary"
            onclick={handleSubmit}
            disabled={!isButtonEnabled}
          >
            {getSubmitButtonText}
          </Button>
        </div>
      {:else}
        <Button
          variant="primary"
          onclick={handleSubmit}
          disabled={!isButtonEnabled}
        >
          {getSubmitButtonText}
        </Button>
      {/if}
    </div>
  {:else}
    <div class="flex items-center justify-center h-full">
      <p class="text-zinc-400">Loading submission details...</p>
    </div>
  {/if}
  <InsetError
    open={showError}
    error={Option.isSome(error) ? error.value : null}
    onClose={() => {
      showError = false;
    }}
  />
</div>
