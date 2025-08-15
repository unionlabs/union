<script lang="ts">
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import InsetError from "$lib/components/model/InsetError.svelte"
import Button from "$lib/components/ui/Button.svelte"
import Label from "$lib/components/ui/Label.svelte"
import * as AppRuntime from "$lib/runtime"
import { getCosmWasmClient } from "$lib/services/cosmos/clients"
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
import { switchChain as cosmosSwitchChain } from "$lib/services/transfer-ucs03-cosmos"
import { getCosmosOfflineSigner } from "$lib/services/transfer-ucs03-cosmos/offline-signer"
import {
  EvmSwitchChainError,
  switchChain as evmSwitchChain,
} from "$lib/services/transfer-ucs03-evm"
import type {
  ConnectorClientError,
  SwitchChainError,
  WaitForTransactionReceiptError,
} from "$lib/services/transfer/errors"
import { transferHashStore } from "$lib/stores/transfer-hash.svelte.ts"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import type { SubmitInstruction } from "$lib/transfer/normal/steps/steps.ts"
import { isValidBech32ContractAddress } from "$lib/utils"
import { Ucs03, ZkgmClientResponse, ZkgmIncomingMessage } from "@unionlabs/sdk"
import { ZkgmClient } from "@unionlabs/sdk"
import { Cosmos, CosmosZkgmClient } from "@unionlabs/sdk-cosmos"
import { Evm, EvmZkgmClient } from "@unionlabs/sdk-evm"
import { GAS_DENOMS } from "@unionlabs/sdk/constants/gas-denoms"
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
import { CryptoError, extractErrorDetails, generateSalt } from "@unionlabs/sdk/utils/index"
import { getTimeoutInNanoseconds24HoursFromNow } from "@unionlabs/sdk/utils/timeout.ts"
import { getPublicClient, http } from "@wagmi/core"
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

const isButtonEnabled = $derived(!isSubmitting || Option.some(error))
let ctaCopy = $state<string>("Submit")

const resetState = () => {
  ctaCopy = "Submit"
  error = Option.none()
  isSubmitting = false
}

const request = $derived(step.instruction)

export const submit = Effect.gen(function*() {
  // if (needsRetry) {
  //   resetState()
  //   return // Exit and let the button click call this function again
  // }

  isSubmitting = true

  error = Option.none()

  const doEvm = Effect.gen(function*() {
    const chain = yield* step.intent.sourceChain.toViemChain()
    const connectorClient = yield* getWagmiConnectorClient

    const publicClient = Evm.PublicClient.Live({
      chain,
      transport: custom(connectorClient),
    })
    const walletClient = Evm.WalletClient.Live({
      account: connectorClient.account,
      chain,
      transport: custom(connectorClient),
    })

    return yield* pipe(
      Effect.sync(() => {
        ctaCopy = "Switching Chain..."
      }),
      Effect.andThen(() => evmSwitchChain(chain)),
      Effect.andThen(() =>
        Effect.sync(() => {
          ctaCopy = "Executing..."
        })
      ),
      Effect.andThen(() => ZkgmClient.execute(request)),
      Effect.andThen((response) =>
        pipe(
          Effect.sync(() => {
            ctaCopy = "Confirming Transaction..."
          }),
          Effect.andThen(() =>
            response.waitFor(
              ZkgmIncomingMessage.LifecycleEvent.$is("EvmTransactionReceiptComplete"),
            )
          ),
          Effect.flatMap(x => Option.map(x, y => y.transactionHash)),
        )
      ),
      Effect.provide(EvmZkgmClient.layerWithoutWallet),
      Effect.provide(publicClient),
      Effect.provide(walletClient),
    )
  })

  const doCosmos = Effect.gen(function*() {
    const chain = step.intent.sourceChain
    const { address } = yield* wallets.getAddressForChain(chain)

    ctaCopy = "Switching Chain..."
    const switchResult = yield* cosmosSwitchChain(chain)

    ctaCopy = "Initializing Signer..."
    const signingClient = yield* getCosmWasmClient(chain)
    const rpcUrl = yield* chain.getRpcUrl("rpc")

    const walletClient = Cosmos.SigningClient.FromSigningClient(
      address,
      signingClient,
    )

    const publicClient = Cosmos.Client.Live(rpcUrl)

    ctaCopy = "Executing..."

    const response = yield* ZkgmClient.execute(step.instruction).pipe(
      Effect.provide(CosmosZkgmClient.layerWithoutSigningClient),
      Effect.provide(walletClient),
      Effect.provide(publicClient),
    )

    return response.txHash
  })

  const sourceChainRpcType = step.intent.sourceChain.rpc_type
  return yield* Match.value(sourceChainRpcType).pipe(
    Match.when("evm", () => doEvm),
    Match.when("cosmos", () => doCosmos),
    Match.orElse(() =>
      Effect.gen(function*() {
        yield* Effect.logFatal("Unknown chain type")
        // TODO: make fail
        return "unknown chain type"
      })
    ),
  )
}).pipe(
  Effect.annotateLogs({
    step: "submit",
  }),
)

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
        console.log("DONE EXECUTING", cause)
        const err = Cause.originalError(cause)
        error = pipe(
          err,
          Cause.failures,
          xs => Array.from(xs),
          Arr.head,
        )
        isSubmitting = false
      },
      onSuccess: (hash) => {
        console.log("TX HASH", hash)
      },
    })
  )
}
</script>

<div class="relative min-w-full p-4 flex flex-col justify-between h-full">
  {#if step && step.intent.sourceChain && step.intent.destinationChain}
    <div class="flex-1 flex flex-col gap-4">
      <h3 class="text-lg font-semibold">Submit Transfer</h3>
      <section>
        <Label>From</Label>
        <ChainComponent chain={step.intent.sourceChain} />
      </section>

      <section>
        <Label>To</Label>
        <ChainComponent chain={step.intent.destinationChain} />
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
        {ctaCopy}
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
