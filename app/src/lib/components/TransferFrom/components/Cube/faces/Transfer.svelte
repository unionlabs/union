<script lang="ts">
import type { ValidationStore } from "$lib/components/TransferFrom/transfer/validation.ts"
import { derived, get, type Readable, writable, type Writable } from "svelte/store"
import type { ContextStore } from "$lib/components/TransferFrom/transfer/context.ts"
import { Button } from "$lib/components/ui/button"
import {
  type EvmChainId,
  createUnionClient,
  evmChainFromChainId,
  type TransferAssetsParameters,
  truncateAddress,
  type AptosBrowserWallet,
  type ChainId,
  http,
  type CosmosChainId
} from "@unionlabs/client"
import { truncate } from "$lib/utilities/format.ts"
import { custom, getConnectorClient, switchChain, waitForTransactionReceipt } from "@wagmi/core"
import { getAddress, type HttpTransport, parseUnits } from "viem"
import { config, userAddrEvm } from "$lib/wallet/evm/config.ts"
import { toast } from "svelte-sonner"
import { aptosStore, getAptosWallet, userAddressAptos } from "$lib/wallet/aptos"
import { stepAfter, stepBefore, type TransferState } from "$lib/transfer/transfer.ts"
import { cosmosStore, getCosmosOfflineSigner, userAddrCosmos } from "$lib/wallet/cosmos"
import { getCosmosChainInfo } from "$lib/wallet/cosmos/chain-info.ts"
import { raise, sleep } from "$lib/utilities"
import { submittedTransfers } from "$lib/stores/submitted-transfers.ts"
import { toIsoString } from "$lib/utilities/date.ts"
import { goto } from "$app/navigation"
import type { CubeFaces } from "$lib/components/TransferFrom/components/Cube/types.ts"
import Stepper from "$lib/components/stepper.svelte"
import type { Step } from "$lib/stepper-types.ts"
import Truncate from "$lib/components/truncate.svelte"

interface Props {
  stores: {
    context: Readable<ContextStore>
    validation: Readable<ValidationStore>
  }
  rotateTo: (face: CubeFaces) => void
}

export let stores: Props["stores"]
export let rotateTo: Props["rotateTo"]

let { validation, context } = stores

const REDIRECT_DELAY_MS = 5000
let transferState: Writable<TransferState> = writable({ kind: "PRE_TRANSFER" })

let confirmed = false

const transfer = async () => {}

const stateToStatus = <K extends TransferState["kind"]>(
  state: TransferState,
  kind: K,
  pendingTitle: string,
  completedTitle: string,
  errorFormatter: (ts: Extract<TransferState, { kind: K }>) => unknown,
  warningFormatter: (ts: Extract<TransferState, { kind: K }>) => unknown,
  progressFormatter: (ts: Extract<TransferState, { kind: K }>) => unknown
) =>
  stepBefore(state, kind)
    ? { status: "PENDING", title: pendingTitle }
    : stepAfter(state, kind)
      ? { status: "COMPLETED", title: completedTitle }
      : // @ts-ignore
        state.warning !== undefined
        ? warningFormatter(state as Extract<TransferState, { kind: K }>)
        : // @ts-ignore
          state.error !== undefined
          ? errorFormatter(state as Extract<TransferState, { kind: K }>)
          : progressFormatter(state as Extract<TransferState, { kind: K }>)

let stepperSteps = derived(
  [context, transferState, validation],
  ([$context, $transferState, $validation]) => {
    if (!$validation.isValid) return []
    if ($transferState.kind === "PRE_TRANSFER") return [] // don"t generate steps before transfer is ready
    if ($validation.transfer.sourceChain?.rpc_type === "evm") {
      // TODO: Refactor this by implementing Ord for transferState
      return [
        // Do not uncomment
        stateToStatus(
          $transferState,
          "SWITCHING_TO_CHAIN",
          `Switch to ${$validation.transfer.sourceChain.display_name}`,
          `Switched to ${$validation.transfer.sourceChain.display_name}`,
          ts => ({
            status: "ERROR",
            title: `Error switching to ${$validation.transfer.sourceChain.display_name}`,
            description: `There was an issue switching to ${$validation.transfer.sourceChain.display_name} to your wallet. ${ts.warning}`
          }),
          () => ({
            status: "WARNING",
            title: `Could not automatically switch chain.`,
            description: `Please make sure your wallet is connected to  ${$validation.transfer.sourceChain.display_name}`
          }),
          () => ({
            status: "IN_PROGRESS",
            title: `Switching to ${$validation.transfer.sourceChain.display_name}`,
            description: `Click "Approve" in wallet.`
          })
        ),
        stateToStatus(
          $transferState,
          "APPROVING_ASSET",
          "Approve ERC20",
          "Approved ERC20",
          ts => ({
            status: "ERROR",
            title: `Error approving ERC20`,
            description: `${ts.error}`
          }),
          () => ({}),
          () => ({
            status: "IN_PROGRESS",
            title: "Approving ERC20",
            description: "Click 'Next' and 'Approve' in wallet."
          })
        ),
        stateToStatus(
          $transferState,
          "AWAITING_APPROVAL_RECEIPT",
          "Wait for approval receipt",
          "Received approval receipt",
          ts => ({
            status: "ERROR",
            title: `Error waiting for approval receipt`,
            description: `${ts.error}`
          }),
          () => ({}),
          () => ({
            status: "IN_PROGRESS",
            title: "Awaiting approval receipt",
            description: `Waiting on ${$validation.transfer.sourceChain.display_name}`
          })
        ),
        stateToStatus(
          $transferState,
          "SIMULATING_TRANSFER",
          "Simulate transfer",
          "Simulated transfer",
          ts => ({
            status: "ERROR",
            title: `Error simulating transfer on ${$validation.transfer.sourceChain.display_name}`,
            // @ts-expect-error
            description: `${ts.error}`
          }),
          () => ({
            status: "WARNING",
            title: `Failed to simulate transfer`,
            description: `You can still attempt to make this transfer in your wallet`
          }),
          () => ({
            status: "IN_PROGRESS",
            title: "Simulating transfer",
            description: `Waiting on ${$validation.transfer.sourceChain.display_name}`
          })
        ),
        stateToStatus(
          $transferState,
          "CONFIRMING_TRANSFER",
          "Confirm transfer",
          "Confirmed transfer",
          ts => ({
            status: "ERROR",
            title: "Error confirming transfer",
            description: `${ts.error}`
          }),
          () => ({}),
          () => ({
            status: "IN_PROGRESS",
            title: "Confirming your transfer",
            description: `Click "Confirm" in your wallet`
          })
        ),
        stateToStatus(
          $transferState,
          "AWAITING_TRANSFER_RECEIPT",
          "Wait for transfer receipt",
          "Confirmed transfer",
          ts => ({
            status: "ERROR",
            title: "Error while waiting on transfer receipt",
            description: `tx hash: ${ts.transferHash}, error: ${ts.error}`
          }),
          () => ({}),
          () => ({
            status: "IN_PROGRESS",
            title: "Awaiting transfer receipt",
            description: `Waiting on ${$validation.transfer.sourceChain.display_name}`
          })
        ),
        stateToStatus(
          $transferState,
          "TRANSFERRING",
          "Transfer assets",
          "Transferred assets",
          () => ({}),
          () => ({}),
          () => ({
            status: "IN_PROGRESS",
            title: "Transferring assets",
            description: `Successfully initiated transfer`
          })
        )
      ] as Array<Step>
    }
    if (
      $validation.transfer.sourceChain?.rpc_type === "cosmos" ||
      $validation.transfer.sourceChain?.rpc_type === "aptos"
    ) {
      return [
        stateToStatus(
          $transferState,
          "SWITCHING_TO_CHAIN",
          `Switch to ${$validation.transfer.sourceChain.display_name}`,
          `Switched to ${$validation.transfer.sourceChain.display_name}`,
          ts => ({
            status: "ERROR",
            title: `Error switching to ${$validation.transfer.sourceChain.display_name}`,
            description: `There was an issue switching to ${$validation.transfer.sourceChain.display_name} to your wallet. ${ts.warning}`
          }),
          () => ({
            status: "WARNING",
            title: `Could not automatically switch chain.`,
            description: `Please make sure your wallet is connected to  ${$validation.transfer.sourceChain.display_name}`
          }),
          () => ({
            status: "IN_PROGRESS",
            title: `Switching to ${$validation.transfer.sourceChain.display_name}`,
            description: `Click "Approve" in wallet.`
          })
        ),
        stateToStatus(
          $transferState,
          "CONFIRMING_TRANSFER",
          "Confirm transfer",
          "Confirmed transfer",
          ts => ({
            status: "ERROR",
            title: "Error confirming transfer",
            description: `${ts.error}`
          }),
          () => ({}),
          () => ({
            status: "IN_PROGRESS",
            title: "Confirming your transfer",
            description: `Click "Approve" in your wallet`
          })
        ),
        stateToStatus(
          $transferState,
          "TRANSFERRING",
          "Transfer assets",
          "Transferred assets",
          () => ({}),
          () => ({}),
          () => ({
            status: "IN_PROGRESS",
            title: "Transferring assets",
            description: `Successfully initiated transfer`
          })
        )
      ] as Array<Step>
    }
    raise("trying to make stepper for unsupported chain")
  }
)
</script>

<div class="h-full w-full flex flex-col justify-between p-4 overflow-y-scroll">
  {#if $validation.isValid && !confirmed}
    <div>
      <div class="flex justify-between">
        <span>{$validation.transfer.sourceChain.display_name}</span>
        <Truncate value={$validation.transfer.sender} type="address"/>
      </div>
      <div class="flex justify-between">
        <span>{$validation.transfer.destinationChain.display_name}</span>
        <Truncate value={$validation.transfer.receiver} type="address"/>
      </div>
      <div class="flex justify-between">
        <span>{$validation.transfer.amount}</span>
        <Truncate value={$validation.transfer.asset.metadata.denom} type="address"/>
      </div>
    </div>
  {/if}

  {#if $validation.transfer?.sourceChain}
    <Stepper
            steps={stepperSteps}
            on:cancel={() => transferState.set({ kind: 'PRE_TRANSFER' })}
            onRetry={() => {
              transferState.update(ts => {
                // @ts-ignore
                ts.error = undefined
                return ts
              })
              transfer()
            }}
    />
  {/if}

  {#if !confirmed}
  <div class="flex flex-col gap-2">
    <Button on:click={transfer}>Confirm</Button>
    <Button variant="outline" on:click={() => rotateTo("intentFace")}>CANCEL</Button>
  </div>
  {/if}
</div>


