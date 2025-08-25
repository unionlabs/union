<script lang="ts">
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import InsetError from "$lib/components/model/InsetError.svelte"
import Button from "$lib/components/ui/Button.svelte"
import Label from "$lib/components/ui/Label.svelte"
import * as AppRuntime from "$lib/runtime"
import { getCosmWasmClient } from "$lib/services/cosmos/clients"
import { getWagmiConnectorClient } from "$lib/services/evm/clients"
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
import { switchChain as evmSwitchChain } from "$lib/services/transfer-ucs03-evm"
import type {
  ConnectorClientError,
  SwitchChainError,
  WaitForTransactionReceiptError,
} from "$lib/services/transfer/errors"
import { transferHashStore } from "$lib/stores/transfer-hash.svelte"
import { wallets } from "$lib/stores/wallets.svelte"
import type { SubmitInstruction } from "$lib/transfer/normal/steps/steps"
import { safeOpts } from "$lib/transfer/shared/services/handlers/safe"
import { getLastConnectedWalletId } from "$lib/wallet/evm/config.svelte"
import { ZkgmClientError, ZkgmIncomingMessage } from "@unionlabs/sdk"
import { ZkgmClient } from "@unionlabs/sdk"
import { Cosmos, CosmosZkgmClient } from "@unionlabs/sdk-cosmos"
import { Evm, EvmZkgmClient, Safe } from "@unionlabs/sdk-evm"
import type { ExecuteContractError } from "@unionlabs/sdk/cosmos"
import {
  CreateViemPublicClientError,
  CreateViemWalletClientError,
  WriteContractError,
} from "@unionlabs/sdk/evm"
import type {
  CosmosAddressEncodeError,
  NotACosmosChainError,
  TransactionHash,
} from "@unionlabs/sdk/schema"
import { CryptoError } from "@unionlabs/sdk/utils/index"
import { Array as Arr, Cause, Effect, Exit, Layer, Match, Option, Stream } from "effect"
import * as B from "effect/Boolean"
import type { NoSuchElementException } from "effect/Cause"
import { pipe } from "effect/Function"
import { custom } from "viem"

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
    | ZkgmClientError.RequestError
    | ZkgmClientError.ResponseError
    | Evm.CreateWalletClientError
    | Evm.CreatePublicClientError
    | Cosmos.ClientError
    | NoSuchElementException
    | CryptoError
    | ExecuteContractError
    | Safe.SafeError
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

const isButtonEnabled = $derived(!isSubmitting || needsRetry)
let ctaCopy = $state<string>("Submit")

const resetState = () => {
  ctaCopy = "Submit"
  error = Option.none()
  isSubmitting = false
}

const request = $derived(step.instruction)

const startPolling = (transactionHash: TransactionHash) => {
  transferHashStore.startPolling(transactionHash)
  onSubmit()
}

export const submit = Effect.gen(function*() {
  if (needsRetry) {
    resetState()
    return Effect.void // Exit and let the button click call this function again
  }

  isSubmitting = true

  error = Option.none()

  const doEvm = Effect.gen(function*() {
    const chain = yield* step.intent.sourceChain.toViemChain()
    const connectorClient = yield* getWagmiConnectorClient

    const usingSafe = getLastConnectedWalletId() === "safe"
    const maybeSafe = B.match(
      getLastConnectedWalletId() === "safe",
      {
        onTrue: () => Safe.Safe.Default(safeOpts),
        onFalse: () => Layer.empty,
      },
    )

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
      Effect.if(usingSafe, {
        onTrue: () => Effect.void,
        onFalse: () =>
          pipe(
            Effect.sync(() => {
              ctaCopy = "Switching Chain..."
            }),
            Effect.andThen(() => evmSwitchChain(chain)),
          ),
      }),
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
            Effect.if(
              usingSafe,
              {
                onFalse: () =>
                  pipe(
                    response.waitFor(
                      ZkgmIncomingMessage.LifecycleEvent.$is("EvmTransactionReceiptComplete"),
                    ),
                    Effect.flatMap(Effect.map(x => x.transactionHash)),
                  ),
                onTrue: () =>
                  pipe(
                    Effect.sync(() => {
                      ctaCopy = "Waiting for Safe..."
                    }),
                    Effect.andThen(() =>
                      pipe(
                        response.waitFor(
                          ZkgmIncomingMessage.LifecycleEvent.$is("WaitForSafeWalletHash"),
                        ),
                        Effect.flatMap(Effect.map(x => x.hash)),
                      )
                    ),
                  ),
              },
            )
          ),
        )
      ),
      Effect.provide(EvmZkgmClient.layerWithoutWallet),
      Effect.provide(maybeSafe),
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

  AppRuntime.runPromiseExit(submit).then(exit =>
    Exit.match(exit, {
      onFailure: cause => {
        const err = Cause.originalError(cause)
        ctaCopy = "Retry"
        error = pipe(
          err,
          Cause.failures,
          xs => Array.from(xs),
          Arr.head,
        )
        isSubmitting = false
      },
      onSuccess: (hash) => {
        startPolling(hash as TransactionHash)
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
