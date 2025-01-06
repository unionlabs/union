<script lang="ts">
import type { IntentsStore } from "$lib/components/TransferFrom/transfer/intents.ts"
import type { ValidationStoreAndMethods } from "$lib/components/TransferFrom/transfer/validation.ts"
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
import { userAddrOnChain } from "$lib/utilities/address.ts"
import { toIsoString } from "$lib/utilities/date.ts"
import { goto } from "$app/navigation"
import type { CubeFaces } from "$lib/components/TransferFrom/components/Cube/types.ts"
import Stepper from "$lib/components/stepper.svelte"
import type { Step } from "$lib/stepper-types.ts"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.ts"

interface Props {
  stores: {
    rawIntents: RawIntentsStore
    intents: Readable<IntentsStore>
    context: Readable<ContextStore>
    validation: ValidationStoreAndMethods
  }
  rotateTo: (face: CubeFaces) => void
}

export let stores: Props["stores"]
export let rotateTo: Props["rotateTo"]

let { intents, context } = stores

const REDIRECT_DELAY_MS = 5000
let transferState: Writable<TransferState> = writable({ kind: "PRE_TRANSFER" })

const transfer = async () => {
  if (!$intents.selectedAsset.address) return toast.error(`Please select a asset`)
  if (!$intents.sourceChain.chain_id) return toast.error("Please select a from chain")
  if (!$intents.sourceChain) return toast.error("can't find chain in config")
  if (!$intents.destinationChain) return toast.error("can't find chain in config")
  if (!$intents.destinationChain.chain_id) return toast.error("Please select a to chain")

  if (!$intents.amount) return toast.error("Please select an amount")
  if ($intents.sourceChain.rpc_type === "evm" && !$context.userAddress.evm)
    return toast.error("No evm wallet connected")
  if ($intents.sourceChain.rpc_type === "cosmos" && !$context.userAddress.cosmos)
    return toast.error("No cosmos wallet connected")
  if ($intents.sourceChain.rpc_type === "aptos" && !$context.userAddress.aptos)
    return toast.error("No aptos wallet connected")

  if (!$intents.receiver) return toast.error("Invalid receiver")

  console.log("click")

  let decimals = $intents.selectedAsset.decimals
  let parsedAmount = parseUnits($intents.amount, decimals)

  /** --- APTOS START --- */
  if ($intents.sourceChain?.rpc_type === "aptos") {
    const { connectedWallet, connectionStatus } = get(aptosStore)
    if ($userAddressAptos === null) return toast.error("No aptos user address found")

    if (connectionStatus !== "connected" || !connectedWallet) {
      transferState.set({
        kind: "SWITCHING_TO_CHAIN",
        warning: new Error("No wallet connected")
      })
      return
    }

    const wallet = getAptosWallet(connectedWallet)
    if (!wallet) {
      transferState.set({
        kind: "SWITCHING_TO_CHAIN",
        warning: new Error(`${connectedWallet} wallet not found`)
      })
      return
    }

    // @ts-ignore
    transferState.set({ kind: "SWITCHING_TO_CHAIN" })

    const rpcUrl = $intents.sourceChain?.rpcs.find(rpc => rpc.type === "rpc")?.url
    if (!rpcUrl) return toast.error(`no rpc available for ${$intents.sourceChain?.display_name}`)

    if (stepBefore($transferState, "CONFIRMING_TRANSFER")) {
      const chainInfo = await wallet.getNetwork()

      if (chainInfo?.chainId.toString() !== $intents.sourceChain.chain_id) {
        transferState.set({
          kind: "SWITCHING_TO_CHAIN",
          warning: new Error("Failed to switch chain")
        })
        return
      }

      // @ts-ignore
      transferState.set({ kind: "CONFIRMING_TRANSFER" })
    }

    if (stepBefore($transferState, "TRANSFERRING")) {
      try {
        const client = createUnionClient({
          chainId: "2",
          account: await wallet?.getAccount(),
          transport: wallet as AptosBrowserWallet
        })

        const transferPayload = {
          simulate: true,
          receiver: $intents.receiver,
          amount: parsedAmount,
          authAccess: "wallet",
          denomAddress: $intents.selectedAsset.address,
          destinationChainId: $intents.destinationChain.chain_id as ChainId
        } satisfies TransferAssetsParameters<"2">

        const transfer = await client.transferAsset(transferPayload)

        if (transfer.isErr()) throw transfer.error
        transferState.set({ kind: "TRANSFERRING", transferHash: transfer.value })
      } catch (error) {
        if (error instanceof Error) {
          // @ts-ignore
          transferState.set({ kind: "CONFIRMING_TRANSFER", error })
        }
        return
      }
    }

    /** --- APTOS END --- */
  } else if ($intents.sourceChain.rpc_type === "cosmos") {
    const { connectedWallet, connectionStatus } = get(cosmosStore)
    if ($userAddrCosmos === null) return toast.error("No Cosmos user address found")

    if (connectionStatus !== "connected" || !connectedWallet) {
      transferState.set({
        kind: "SWITCHING_TO_CHAIN",
        warning: new Error("No wallet connected")
      })
      return
    }

    const wallet = window[connectedWallet]

    if (!wallet) {
      transferState.set({
        kind: "SWITCHING_TO_CHAIN",
        warning: new Error(`${connectedWallet} wallet not found`)
      })
      return
    }

    // @ts-ignore
    transferState.set({ kind: "SWITCHING_TO_CHAIN" })

    const rpcUrl = $intents.sourceChain.rpcs.find(rpc => rpc.type === "rpc")?.url
    if (!rpcUrl) return toast.error(`no rpc available for ${$intents.sourceChain.display_name}`)

    if (stepBefore($transferState, "CONFIRMING_TRANSFER")) {
      const chainInfo = getCosmosChainInfo($intents.sourceChain.chain_id, connectedWallet)

      if (chainInfo === null) {
        transferState.set({
          kind: "SWITCHING_TO_CHAIN",
          warning: new Error("Failed to switch chain")
        })
        return
      }

      try {
        await wallet.experimentalSuggestChain(chainInfo)
        await wallet.enable([$intents.sourceChain.chain_id])
      } catch (error) {
        if (error instanceof Error) {
          transferState.set({
            kind: "SWITCHING_TO_CHAIN",
            warning: error
          })
        } else {
          transferState.set({
            kind: "SWITCHING_TO_CHAIN",
            warning: new Error("invalid error")
          })
        }
        return
      }
      // @ts-ignore
      transferState.set({ kind: "CONFIRMING_TRANSFER" })
    }

    if (stepBefore($transferState, "TRANSFERRING")) {
      try {
        const cosmosOfflineSigner = await getCosmosOfflineSigner({
          connectedWallet,
          chainId: $intents.sourceChain.chain_id
        })
        const unionClient = createUnionClient({
          account: cosmosOfflineSigner,
          transport: http(`https://${rpcUrl}`),
          chainId: $intents.sourceChain.chain_id as CosmosChainId,
          gasPrice: { amount: "0.0025", denom: $intents.selectedAsset.address }
        })

        const transfer = await unionClient.transferAsset({
          autoApprove: true,
          receiver: $intents.receiver,
          amount: parsedAmount,
          denomAddress: $intents.selectedAsset.address,
          account: cosmosOfflineSigner,
          // TODO: verify chain id is correct
          destinationChainId: $intents.destinationChain.chain_id as ChainId,
          gasPrice: { amount: "0.0025", denom: $intents.selectedAsset.address }
        })
        if (transfer.isErr()) throw transfer.error
        transferState.set({ kind: "TRANSFERRING", transferHash: transfer.value })
      } catch (error) {
        if (error instanceof Error) {
          // @ts-ignore
          transferState.set({ kind: "CONFIRMING_TRANSFER", error })
        }
        return
      }
    }
  } else if ($intents.sourceChain.rpc_type === "evm") {
    const connectorClient = await getConnectorClient(config)
    const selectedChain = evmChainFromChainId($intents.sourceChain.chain_id)

    const unionClient = createUnionClient({
      account: connectorClient.account,
      chainId: $intents.sourceChain.chain_id as EvmChainId,
      transport: custom(window.ethereum) as unknown as HttpTransport
    })

    if (!selectedChain) {
      toast.error("From chain not found or supported")
      return
    }

    if ($userAddrEvm === null) return toast.error("No Cosmos user address found")

    if (window.ethereum === undefined) raise("no ethereum browser extension")

    if (stepBefore($transferState, "SWITCHING_TO_CHAIN")) {
      transferState.set({ kind: "SWITCHING_TO_CHAIN" })
    }

    if ($transferState.kind === "SWITCHING_TO_CHAIN") {
      if ($transferState.warning) {
        transferState.set({ kind: "APPROVING_ASSET" })
        transfer()
        return
      }
      // ^ the user is continuing continuing after having seen the warning

      try {
        await switchChain(config, { chainId: selectedChain.id })
      } catch (error) {
        if (error instanceof Error) {
          transferState.set({ kind: "SWITCHING_TO_CHAIN", warning: error })
        }
        return
      }
      transferState.set({ kind: "APPROVING_ASSET" })
    }

    if ($transferState.kind === "APPROVING_ASSET") {
      let hash: `0x${string}` | null = null

      try {
        const approve = await unionClient.approveTransaction({
          amount: parsedAmount,
          receiver: $intents.receiver,
          denomAddress: getAddress($intents.selectedAsset.address),
          // TODO: verify chain id is correct
          destinationChainId: $intents.destinationChain.chain_id as ChainId
        })

        if (approve.isErr()) throw approve.error
        hash = approve.value
      } catch (error) {
        if (error instanceof Error) {
          transferState.set({ kind: "APPROVING_ASSET", error })
        }
        return
      }
      transferState.set({ kind: "AWAITING_APPROVAL_RECEIPT", hash })
    }

    if ($transferState.kind === "AWAITING_APPROVAL_RECEIPT") {
      try {
        await waitForTransactionReceipt(config, { hash: $transferState.hash })
      } catch (error) {
        if (error instanceof Error) {
          transferState.set({
            kind: "AWAITING_APPROVAL_RECEIPT",
            hash: $transferState.hash,
            error
          })
        }
        return
      }
      transferState.set({ kind: "SIMULATING_TRANSFER" })
    }

    if ($transferState.kind === "SIMULATING_TRANSFER") {
      console.log("simulating transfer step")

      if ($transferState.warning) {
        transferState.set({ kind: "CONFIRMING_TRANSFER", contractRequest: null })
        transfer()
        return
      }

      // ^ the user is continuing continuing after having seen the warning

      console.log("confirming transfers test")

      try {
        transferState.set({ kind: "CONFIRMING_TRANSFER", contractRequest: null })
      } catch (error) {
        if (error instanceof Error) {
          transferState.set({ kind: "SIMULATING_TRANSFER", warning: error })
        }
        return
      }
    }

    if ($transferState.kind === "CONFIRMING_TRANSFER") {
      try {
        const transfer = await unionClient.transferAsset({
          autoApprove: false,
          amount: parsedAmount,
          receiver: $intents.receiver,
          denomAddress: getAddress($intents.selectedAsset.address),
          // TODO: verify chain id is correct
          destinationChainId: $intents.destinationChain.chain_id as ChainId
        })
        if (transfer.isErr()) throw transfer.error
        transferState.set({ kind: "AWAITING_TRANSFER_RECEIPT", transferHash: transfer.value })
      } catch (error) {
        if (error instanceof Error) {
          transferState.set({
            kind: "CONFIRMING_TRANSFER",
            contractRequest: $transferState.contractRequest,
            error
          })
        }
      }
    }

    if ($transferState.kind === "AWAITING_TRANSFER_RECEIPT") {
      try {
        await waitForTransactionReceipt(config, {
          hash: $transferState.transferHash
        })
        transferState.set({ kind: "TRANSFERRING", transferHash: $transferState.transferHash })
      } catch (error) {
        if (error instanceof Error) {
          transferState.set({
            kind: "AWAITING_TRANSFER_RECEIPT",
            transferHash: $transferState.transferHash,
            error
          })
        }
      }
    }
  } else {
    console.error("invalid rpc type")
  }

  if ($transferState.kind === "TRANSFERRING") {
    await sleep(REDIRECT_DELAY_MS)
    submittedTransfers.update(ts => {
      // @ts-ignore
      ts[$transferState.transferHash] = {
        source_chain_id: $intents.sourceChain.chain_id,
        destination_chain_id: $intents.destinationChain?.chain_id,
        source_transaction_hash: $transferState.transferHash,
        hop_chain_id: $intents.destinationChain?.chain_id,
        sender: userAddrOnChain($context.userAddress, $intents.sourceChain),
        normalized_sender:
          $intents.sourceChain?.rpc_type === "cosmos"
            ? $userAddrCosmos?.normalized
            : $userAddrEvm?.normalized,
        transfer_day: toIsoString(new Date(Date.now())).split("T")[0],
        receiver: $intents.receiver,
        assets: {
          [$intents.selectedAsset.address]: {
            info:
              $intents.sourceChain?.assets?.find(d => d.denom === $intents.selectedAsset.address) ??
              null,
            amount: parsedAmount
          }
        },
        amount: parsedAmount
      }
      return ts
    })
    goto(`/explorer/transfers/${$transferState.transferHash}`)
  }
}

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

let stepperSteps = derived([context, transferState], ([$context, $transferState]) => {
  if ($transferState.kind === "PRE_TRANSFER") return [] // don"t generate steps before transfer is ready
  if ($intents.sourceChain?.rpc_type === "evm") {
    // TODO: Refactor this by implementing Ord for transferState
    return [
      // Do not uncomment
      stateToStatus(
        $transferState,
        "SWITCHING_TO_CHAIN",
        `Switch to ${$intents.sourceChain.display_name}`,
        `Switched to ${$intents.sourceChain.display_name}`,
        ts => ({
          status: "ERROR",
          title: `Error switching to ${$intents.sourceChain.display_name}`,
          description: `There was an issue switching to ${$intents.sourceChain.display_name} to your wallet. ${ts.warning}`
        }),
        () => ({
          status: "WARNING",
          title: `Could not automatically switch chain.`,
          description: `Please make sure your wallet is connected to  ${$intents.sourceChain.display_name}`
        }),
        () => ({
          status: "IN_PROGRESS",
          title: `Switching to ${$intents.sourceChain.display_name}`,
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
          description: `Waiting on ${$intents.sourceChain.display_name}`
        })
      ),
      stateToStatus(
        $transferState,
        "SIMULATING_TRANSFER",
        "Simulate transfer",
        "Simulated transfer",
        ts => ({
          status: "ERROR",
          title: `Error simulating transfer on ${$intents.sourceChain.display_name}`,
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
          description: `Waiting on ${$intents.sourceChain.display_name}`
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
          description: `Waiting on ${$intents.sourceChain.display_name}`
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
  if ($intents.sourceChain?.rpc_type === "cosmos" || $intents.sourceChain?.rpc_type === "aptos") {
    return [
      stateToStatus(
        $transferState,
        "SWITCHING_TO_CHAIN",
        `Switch to ${$intents.sourceChain.display_name}`,
        `Switched to ${$intents.sourceChain.display_name}`,
        ts => ({
          status: "ERROR",
          title: `Error switching to ${$intents.sourceChain.display_name}`,
          description: `There was an issue switching to ${$intents.sourceChain.display_name} to your wallet. ${ts.warning}`
        }),
        () => ({
          status: "WARNING",
          title: `Could not automatically switch chain.`,
          description: `Please make sure your wallet is connected to  ${$intents.sourceChain.display_name}`
        }),
        () => ({
          status: "IN_PROGRESS",
          title: `Switching to ${$intents.sourceChain.display_name}`,
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
})
</script>

<div class="h-full w-full flex flex-col justify-between p-4 overflow-y-scroll">
  <div>
    <h2>Transfer</h2>
    <p>RPC_TYPE: {$intents?.sourceChain?.rpc_type}</p>
    <p>SOURCE: {$intents?.sourceChain?.display_name}</p>
    <p>DESTINATION: {$intents?.destinationChain?.display_name}</p>
    <p>ASSET: {$intents?.selectedAsset?.address ? truncate($intents.selectedAsset.address, 12) : ""}</p>
    <p>AMOUNT: {$intents.amount}</p>
    <p>RECEIVER: {truncateAddress({address: $intents.receiver})}</p>
  </div>

  {#if $intents.sourceChain}
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

  <div class="flex flex-col gap-2">
    <Button on:click={transfer}>Confirm</Button>
    <Button variant="outline" on:click={() => rotateTo("intentFace")}>CANCEL</Button>
  </div>
</div>