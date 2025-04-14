<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import { lockedTransferStore } from "../locked-transfer.svelte.ts"
import { Array as Arr, Cause, Effect, Exit, Match, Option, Struct } from "effect"
import {
  hasFailedExit as evmHasFailedExit,
  isComplete as evmIsComplete,
  nextStateEvm,
  TransactionSubmissionEvm
} from "$lib/components/Transfer/state/evm.ts"
import { generateSalt } from "@unionlabs/sdk/utils"
import { getConnectorClient, type GetConnectorClientErrorType, http } from "@wagmi/core"
import { createViemPublicClient, createViemWalletClient } from "@unionlabs/sdk/evm"
import { ConnectorClientError } from "$lib/services/transfer"
import { wagmiConfig } from "$lib/wallet/evm/wagmi-config.ts"
import { custom, encodeAbiParameters, fromHex } from "viem"
import { ucs03ZkgmAbi } from "$lib/abi/ucs03.ts"
import {
  hasFailedExit as cosmosHasFailedExit,
  isComplete as cosmosIsComplete,
  nextStateCosmos,
  TransactionSubmissionCosmos
} from "$lib/components/Transfer/state/cosmos.ts"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import { getCosmWasmClient } from "$lib/services/cosmos/clients.ts"
import { cosmosStore } from "$lib/wallet/cosmos"
import { instructionAbi } from "@unionlabs/sdk/evm/abi"
import { encodeAbi } from "@unionlabs/sdk/ucs03/instruction.ts"
import { transferHashStore } from "$lib/stores/transfer-hash.svelte.ts"
import { isValidBech32ContractAddress } from "$lib/utils"
import { is } from "../transfer-step.ts"
import Label from "$lib/components/ui/Label.svelte"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import { getTimeoutInNanoseconds24HoursFromNow } from "@unionlabs/sdk/utils/timeout.ts"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"

type Props = {
  stepIndex: number
  onSubmit: () => void
  onCancel?: () => void
  actionButtonText: string
}

const { stepIndex, onSubmit, onCancel, actionButtonText }: Props = $props()

const lts = lockedTransferStore.get()

const step = $derived(
  lts.pipe(
    Option.map(Struct.get("steps")),
    Option.flatMap(Arr.get(stepIndex)),
    Option.filter(is("SubmitInstruction"))
  )
)

const sourceChain = $derived(lts.pipe(Option.map(Struct.get("sourceChain"))))
const destinationChain = $derived(lts.pipe(Option.map(Struct.get("destinationChain"))))

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

// Function to reset state for retry
const resetState = () => {
  ets = TransactionSubmissionEvm.Filling()
  cts = TransactionSubmissionCosmos.Filling()
  error = Option.none()
  isSubmitting = false
}

function normalizeError(cause: unknown): { _tag: string; cause: string } {
  const raw = Cause.squash(cause)
  if (typeof raw === "string") return { _tag: "UnknownError", cause: raw }
  if (raw instanceof Error) return { _tag: raw.name, cause: raw.message }
  if (typeof raw === "object" && raw !== null && "cause" in raw && typeof raw.cause === "string") {
    return raw as { _tag: string; cause: string }
  }
  return { _tag: "UnknownError", cause: "An unknown error occurred" }
}

export const submit = Effect.gen(function* () {
  if (Option.isNone(step) || Option.isNone(lts)) return

  // If we're retrying, reset everything first
  if (needsRetry) {
    resetState()
    return // Exit and let the button click call this function again
  }

  // Set submitting state
  isSubmitting = true
  error = Option.none()

  try {
    const sourceChainRpcType = lts.value.sourceChain.rpc_type

    yield* Match.value(sourceChainRpcType).pipe(
      Match.when("evm", () =>
        Effect.gen(function* () {
          const viemChain = lts.value.sourceChain.toViemChain()
          if (Option.isNone(viemChain)) return Effect.succeed(null)

          const publicClient = yield* createViemPublicClient({
            chain: viemChain.value,
            transport: http()
          })

          const connectorClient = yield* Effect.tryPromise({
            try: () => getConnectorClient(wagmiConfig),
            catch: err =>
              new ConnectorClientError({
                cause: err as GetConnectorClientErrorType
              })
          })

          const walletClient = yield* createViemWalletClient({
            account: connectorClient.account,
            chain: viemChain.value,
            transport: custom(connectorClient)
          })

          do {
            const timeoutTimestamp = getTimeoutInNanoseconds24HoursFromNow()
            const salt = yield* generateSalt("evm")
            ets = yield* Effect.promise(() =>
              nextStateEvm(ets, viemChain.value, publicClient, walletClient, {
                chain: viemChain.value,
                account: connectorClient.account,
                address: lts.value.channel.source_port_id,
                abi: ucs03ZkgmAbi,
                functionName: "send",
                args: [
                  lts.value.channel.source_channel_id,
                  0n,
                  timeoutTimestamp,
                  salt,
                  {
                    opcode: step.value.instruction.opcode,
                    version: step.value.instruction.version,
                    operand: encodeAbi(step.value.instruction)
                  }
                ]
              })
            )

            if (ets._tag === "SwitchChainComplete" || ets._tag === "WriteContractComplete") {
              yield* Exit.matchEffect(ets.exit, {
                onFailure: cause =>
                  Effect.sync(() => {
                    error = Option.some(Cause.squash(cause))
                    console.log(error)
                  }),
                onSuccess: () =>
                  Effect.sync(() => {
                    error = Option.none()
                  })
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
            lts.value.sourceChain,
            cosmosStore.connectedWallet
          )

          const sender = yield* lts.value.sourceChain.getDisplayAddress(wallets.cosmosAddress.value)
          const isNative = !isValidBech32ContractAddress(
            fromHex(lts.value.baseToken.denom, "string")
          )

          do {
            const timeout_timestamp = getTimeoutInNanoseconds24HoursFromNow().toString()
            const salt = yield* generateSalt("cosmos")
            cts = yield* Effect.promise(() =>
              nextStateCosmos(
                cts,
                lts.value.sourceChain,
                signingClient,
                sender,
                fromHex(lts.value.channel.source_port_id, "string"),
                {
                  send: {
                    channel_id: lts.value.channel.source_channel_id,
                    timeout_height: "0",
                    timeout_timestamp,
                    salt,
                    instruction: encodeAbiParameters(instructionAbi, [
                      step.value.instruction.version,
                      step.value.instruction.opcode,
                      encodeAbi(step.value.instruction)
                    ])
                  }
                },
                isNative
                  ? [
                      {
                        denom: fromHex(lts.value.baseToken.denom, "string"),
                        amount: step.value.intents[0].baseAmount.toString()
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
                onSuccess: () =>
                  Effect.sync(() => {
                    error = Option.none()
                  })
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
          error = Option.some({
            _tag: "UnknownError",
            cause: "Unsupported chain type"
          })
          return Effect.succeed("unknown chain type")
        })
      )
    )
  } finally {
    // Reset submitting state when done, regardless of success/failure
    isSubmitting = false
  }
})

const handleSubmit = () => {
  // If we need to retry, just reset the state
  if (needsRetry) {
    resetState()
    return
  }

  // Otherwise proceed with the normal submission
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
  {#if Option.isSome(error)}
    {@const _error = error.value}
    <div class="absolute bottom-0 left-0 right-0">
      <ErrorComponent
        class="absolute bottom-0 left-0 right-0"
        error={_error}
        onClose={() => {
          error = Option.none();
        }}
      />
    </div>
  {/if}
  {#if Option.isSome(step) && Option.isSome(sourceChain) && Option.isSome(destinationChain)}
    <div class="flex-1 flex flex-col gap-4">
      <h3 class="text-lg font-semibold">Submit Transfer</h3>
      <section>
        <Label>From</Label>
        <ChainComponent chain={sourceChain.value} />
      </section>

      <section>
        <Label>To</Label>
        <ChainComponent chain={destinationChain.value} />
      </section>
      <p class="text-sm text-zinc-400">
        This will initiate the transfer on
        <ChainComponent chain={sourceChain.value} />. You'll need to confirm the
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
      <Button
        variant="primary"
        onclick={handleSubmit}
        disabled={!isButtonEnabled}
      >
        {getSubmitButtonText}
      </Button>
    </div>
  {:else}
    <div class="flex items-center justify-center h-full">
      <p class="text-zinc-400">Loading submission details...</p>
    </div>
  {/if}
</div>
