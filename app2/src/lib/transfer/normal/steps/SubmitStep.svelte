<script lang="ts">
import { ucs03ZkgmAbi } from "$lib/abi/ucs03.ts"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import InsetError from "$lib/components/model/InsetError.svelte"
import Button from "$lib/components/ui/Button.svelte"
import Label from "$lib/components/ui/Label.svelte"
import { getWagmiConnectorClient } from "$lib/services/evm/clients.ts"
import type {
  CosmosSwitchChainError,
  CosmosWalletNotConnectedError,
  CosmosWalletNotOnWindowError,
  CosmWasmError,
  GasPriceError,
  GetChainInfoError,
  NoCosmosChainInfoError,
  OfflineSignerError,
} from "$lib/services/transfer-ucs03-cosmos"
import type {
  ConnectorClientError,
  SwitchChainError,
  WaitForTransactionReceiptError,
} from "$lib/services/transfer/errors"
import { transferHashStore } from "$lib/stores/transfer-hash.svelte.ts"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import type { SubmitInstruction } from "$lib/transfer/normal/steps/steps.ts"
import * as WriteCosmos from "$lib/transfer/shared/services/write-cosmos.ts"
import * as WriteEvm from "$lib/transfer/shared/services/write-evm.ts"
import { isValidBech32ContractAddress } from "$lib/utils"
import type { ExecuteContractError } from "@unionlabs/sdk/cosmos"
import {
  createViemPublicClient,
  CreateViemPublicClientError,
  createViemWalletClient,
  CreateViemWalletClientError,
  WriteContractError,
} from "@unionlabs/sdk/evm"
import { instructionAbi } from "@unionlabs/sdk/evm/abi"
import type {
  CosmosAddressEncodeError,
  NotACosmosChainError,
  TransactionHash,
} from "@unionlabs/sdk/schema"
import { encodeAbi } from "@unionlabs/sdk/ucs03/instruction.ts"
import { CryptoError, extractErrorDetails, generateSalt } from "@unionlabs/sdk/utils"
import { getTimeoutInNanoseconds24HoursFromNow } from "@unionlabs/sdk/utils/timeout.ts"
import { http } from "@wagmi/core"
import { Array as Arr, Cause, Effect, Exit, Match, Option, Predicate, Unify } from "effect"
import { not } from "effect/Boolean"
import type { NoSuchElementException } from "effect/Cause"
import { compose, constVoid, flow, pipe } from "effect/Function"
import { custom, encodeAbiParameters, fromHex } from "viem"

type Props = {
  stepIndex: number
  step: SubmitInstruction
  onSubmit: () => void
  cancel?: () => void
  actionButtonText: string
}

const { stepIndex, step, onSubmit, cancel, actionButtonText }: Props = $props()

let showError = $state(false)
let ets = $state<WriteEvm.TransactionState>(WriteEvm.TransactionState.Filling())
let cts = $state<WriteCosmos.TransactionState>(WriteCosmos.TransactionState.Filling())
let error = $state<
  Option.Option<
    | ConnectorClientError
    | CosmWasmError
    | CosmosAddressEncodeError
    | CosmosSwitchChainError
    | CosmosWalletNotConnectedError
    | CosmosWalletNotOnWindowError
    | CreateViemPublicClientError
    | CreateViemWalletClientError
    | CryptoError
    | ExecuteContractError
    | GasPriceError
    | GetChainInfoError
    | NoCosmosChainInfoError
    | NoSuchElementException
    | NotACosmosChainError
    | OfflineSignerError
    | SwitchChainError
    | WaitForTransactionReceiptError
    | WriteContractError
  >
>(Option.none())
let isSubmitting = $state(false)

const needsRetry = $derived(Option.isSome(error))

const isButtonEnabled = $derived.by(() => {
  const isFilling = WriteEvm.is("Filling")(ets) || WriteCosmos.is("Filling")(cts)
  const hasError = Option.isSome(error)
  return !isSubmitting && isFilling || hasError
})

const submitButtonText = $derived.by(() => {
  if (Option.isSome(error)) {
    return "Try Again"
  }

  if (!WriteEvm.is("Filling")(ets)) {
    return WriteEvm.toCtaText(actionButtonText)(ets)
  }

  if (!WriteCosmos.is("Filling")(cts)) {
    return WriteCosmos.toCtaText(actionButtonText)(cts)
  }

  return actionButtonText
})

const resetState = () => {
  ets = WriteEvm.TransactionState.Filling()
  cts = WriteCosmos.TransactionState.Filling()
  error = Option.none()
  isSubmitting = false
}

export const submit = Effect.gen(function*() {
  if (needsRetry) {
    resetState()
    return // Exit and let the button click call this function again
  }

  // Set submitting state
  isSubmitting = true
  error = Option.none()

  const startPolling = (transactionHash: TransactionHash) =>
    Effect.sync(() => {
      console.log("GOT TRANSACTION HASH:", transactionHash)
      transferHashStore.startPolling(transactionHash)
      onSubmit()
    })

  const doEvm = Effect.gen(function*() {
    const viemChain = yield* step.intent.sourceChain.toViemChain()
    const publicClient = yield* createViemPublicClient({
      chain: viemChain,
      transport: http(),
    })
    const connectorClient = yield* getWagmiConnectorClient
    const walletClient = yield* createViemWalletClient({
      account: connectorClient.account,
      chain: viemChain,
      transport: custom(connectorClient),
    })
    const timeoutTimestamp = getTimeoutInNanoseconds24HoursFromNow()
    const salt = yield* generateSalt("evm")

    const setEts = (nextEts: typeof ets) =>
      Effect.sync(() => {
        console.log(`ETS transitioning: ${ets._tag} -> ${nextEts._tag}`)
        ets = nextEts
      })

    const nextState = Effect.tap(
      Effect.suspend(() =>
        WriteEvm.nextState(ets, viemChain, publicClient, walletClient, {
          chain: viemChain,
          account: connectorClient.account,
          address: step.intent.channel.source_port_id,
          abi: ucs03ZkgmAbi,
          functionName: "send",
          args: [
            step.intent.channel.source_channel_id,
            0n,
            timeoutTimestamp,
            salt,
            {
              opcode: step.instruction.opcode,
              version: step.instruction.version,
              operand: encodeAbi(step.instruction),
            },
          ],
        })
      ),
      setEts,
    )

    yield* pipe(
      nextState,
      Effect.repeat({ until: WriteEvm.is("TransactionReceiptComplete") }),
      // TODO: remove cast
      Effect.andThen(({ exit }) => startPolling(exit.transactionHash as TransactionHash)),
    )
  })

  const doCosmos = Effect.gen(function*() {
    const walletCosmosAddress = yield* wallets.cosmosAddress
    const sender = yield* step.intent.sourceChain.getDisplayAddress(walletCosmosAddress)
    const isNative = !isValidBech32ContractAddress(step.intent.baseToken)
    const baseToken = step.intent.baseToken === "xion" ? "uxion" : step.intent.baseToken
    const timeout_timestamp = getTimeoutInNanoseconds24HoursFromNow().toString()
    const salt = yield* generateSalt("cosmos")

    const setCts = (nextCts: typeof cts) =>
      Effect.sync(() => {
        console.log(`CTS transitioning: ${cts._tag} -> ${nextCts._tag}`)
        cts = nextCts
      })

    const nextState = Effect.tap(
      Effect.suspend(() =>
        WriteCosmos.nextState(
          cts,
          step.intent.sourceChain,
          sender,
          fromHex(step.intent.channel.source_port_id, "string"),
          {
            send: {
              channel_id: step.intent.channel.source_channel_id,
              timeout_height: "0",
              timeout_timestamp,
              salt,
              instruction: encodeAbiParameters(instructionAbi, [
                step.instruction.version,
                step.instruction.opcode,
                encodeAbi(step.instruction),
              ]),
            },
          },
          isNative
            ? [
              {
                denom: baseToken,
                amount: step.intent.baseAmount.toString(),
              },
            ]
            : undefined,
        )
      ),
      setCts,
    )

    yield* pipe(
      nextState,
      Effect.repeat({ until: WriteCosmos.is("WriteContractComplete") }),
      Effect.andThen(({ exit }) =>
        // TODO: remove cast
        startPolling(`0x${exit.transactionHash}` as TransactionHash)
      ),
    )
  })

  const sourceChainRpcType = step.intent.sourceChain.rpc_type
  yield* Match.value(sourceChainRpcType).pipe(
    Match.when("evm", () => doEvm),
    Match.when("cosmos", () => doCosmos),
    Match.orElse(() =>
      Effect.gen(function*() {
        yield* Effect.logFatal("Unknown chain type")
        // TODO: make fail
        return Effect.succeed("unknown chain type")
      })
    ),
  )

  yield* Effect.sync(() => {
    isSubmitting = false
  })
})

const handleSubmit = () => {
  error = Option.none()
  showError = false
  if (needsRetry) {
    resetState()
    return
  }

  Effect.runPromiseExit(submit).then(exit =>
    Exit.match(exit, {
      onFailure: cause => {
        const err = Cause.originalError(cause)
        Effect.runSync(Effect.logError(cause))
        error = pipe(
          err,
          Cause.failures,
          xs => Array.from(xs),
          Arr.head,
        )
        isSubmitting = false
      },
      onSuccess: constVoid,
    })
  )
}
</script>

<div class="relative min-w-full p-4 flex flex-col justify-between h-full">
  {#if step && step.intent.sourceChain && step.intent.destinationChain}
    <div class="flex-1 flex flex-col gap-4">
      <h3 class="text-lg font-semibold">Submit Transfer</h3>
      <section>
        <Label>From</Label> <ChainComponent chain={step.intent.sourceChain} />
      </section>

      <section>
        <Label>To</Label> <ChainComponent chain={step.intent.destinationChain} />
      </section>
      <p class="text-sm text-zinc-400">
        This will initiate the transfer on
        <ChainComponent chain={step.intent.sourceChain} />. You'll need to confirm the transfer in
        your wallet.
      </p>
    </div>

    <div class="flex justify-between mt-4">
      <Button
        variant="secondary"
        onclick={cancel}
        disabled={!isButtonEnabled}
      >
        Cancel
      </Button>
      <Button
        variant="primary"
        onclick={handleSubmit}
        disabled={!isButtonEnabled}
      >
        {submitButtonText}
      </Button>
    </div>
    {#if Option.isSome(error)}
      <div class="h-2"></div>
      <ErrorComponent
        onOpen={() => {
          showError = true
        }}
        error={error.value}
      />
    {/if}
  {:else}
    <div class="flex items-center justify-center h-full">
      <p class="text-zinc-400">Loading submission details...</p>
    </div>
  {/if}

  <InsetError
    open={showError}
    error={Option.isSome(error) ? error.value : null}
    onClose={() => {
      showError = false
    }}
  />
</div>
