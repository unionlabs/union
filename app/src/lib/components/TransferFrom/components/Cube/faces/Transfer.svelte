<script lang="ts">
import type { ValidationStore } from "$lib/components/TransferFrom/transfer/validation.ts"
import ArrowRightIcon from "virtual:icons/lucide/arrow-right"
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
import { type Chain, type Ucs03Channel } from "$lib/types"
import Token from "$lib/components/token.svelte"
import Address from "$lib/components/address.svelte"

export let chains: Array<Chain>
export let channel: Ucs03Channel
export let transferArgs: {
  baseToken: string
  baseAmount: bigint
  quoteToken: string
  quoteAmount: bigint
  receiver: string
  sourceChannelId: number
  ucs03address: string
}

const REDIRECT_DELAY_MS = 5000
let transferState: Writable<TransferState> = writable({ kind: "PRE_TRANSFER" })

let confirmed = false

const transfer = async () => {
  confirmed = true

  const sourceChain = chains.find(c => c.chain_id === channel.source_chain_id)
  if (!sourceChain) return toast.error("no source chain found")

  const destChain = chains.find(c => c.chain_id === channel.destination_chain_id)
  if (!destChain) return toast.error("no destination chain found")

  // let parsedAmount = parseUnits(
  //   $validation.transfer.amount,
  //   $validation.transfer.asset.metadata.decimals
  // )

  /** --- APTOS START --- */
  //  if ($validation.transfer.sourceChain.rpc_type === "aptos") {
  //    const { connectedWallet, connectionStatus } = get(aptosStore)
  //    if ($userAddressAptos === null) return toast.error("No aptos user address found")
  //
  //    if (connectionStatus !== "connected" || !connectedWallet) {
  //      transferState.set({
  //        kind: "SWITCHING_TO_CHAIN",
  //        warning: new Error("No wallet connected")
  //      })
  //      return
  //    }
  //
  //    const wallet = getAptosWallet(connectedWallet)
  //    if (!wallet) {
  //      transferState.set({
  //        kind: "SWITCHING_TO_CHAIN",
  //        warning: new Error(`${connectedWallet} wallet not found`)
  //      })
  //      return
  //    }
  //
  //    // @ts-ignore
  //    transferState.set({ kind: "SWITCHING_TO_CHAIN" })
  //
  //    const rpcUrl = $validation.transfer.sourceChain?.rpcs.find(rpc => rpc.type === "rpc")?.url
  //    if (!rpcUrl)
  //      return toast.error(`no rpc available for ${$validation.transfer.sourceChain?.display_name}`)
  //
  //    if (stepBefore($transferState, "CONFIRMING_TRANSFER")) {
  //      const chainInfo = await wallet.getNetwork()
  //
  //      if (chainInfo?.chainId.toString() !== $validation.transfer.sourceChain.chain_id) {
  //        transferState.set({
  //          kind: "SWITCHING_TO_CHAIN",
  //          warning: new Error("Failed to switch chain")
  //        })
  //        return
  //      }
  //
  //      // @ts-ignore
  //      transferState.set({ kind: "CONFIRMING_TRANSFER" })
  //    }
  //
  //    if (stepBefore($transferState, "TRANSFERRING")) {
  //      try {
  //        const client = createUnionClient({
  //          chainId: "2",
  //          account: await wallet?.getAccount(),
  //          transport: wallet as AptosBrowserWallet
  //        })
  //
  //        const transferPayload = {
  //          simulate: true,
  //          receiver: $validation.transfer.receiver,
  //          amount: parsedAmount,
  //          authAccess: "wallet",
  //          denomAddress: $validation.transfer.asset?.balance,
  //          destinationChainId: $validation.transfer.destinationChain.chain_id as ChainId
  //        } satisfies TransferAssetsParameters<"2">
  //
  //        const transfer = await client.transferAsset(transferPayload)
  //
  //        if (transfer.isErr()) throw transfer.error
  //        transferState.set({ kind: "TRANSFERRING", transferHash: transfer.value })
  //      } catch (error) {
  //        if (error instanceof Error) {
  //          // @ts-ignore
  //          transferState.set({ kind: "CONFIRMING_TRANSFER", error })
  //        }
  //        return
  //      }
  //    }
  //
  //    /** --- APTOS END --- */
  //    /** --- COSOS START --- */
  if (sourceChain.rpc_type === "cosmos") {
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

    const rpcUrl = sourceChain.rpcs.find(rpc => rpc.type === "rpc")?.url
    if (!rpcUrl) return toast.error(`no rpc available for ${sourceChain.display_name}`)

    if (stepBefore($transferState, "CONFIRMING_TRANSFER")) {
      const chainInfo = getCosmosChainInfo(sourceChain.chain_id, connectedWallet)

      if (chainInfo === null) {
        transferState.set({
          kind: "SWITCHING_TO_CHAIN",
          warning: new Error("Failed to switch chain")
        })
        return
      }

      try {
        await wallet.experimentalSuggestChain(chainInfo)
        await wallet.enable([sourceChain.chain_id])
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
          chainId: sourceChain.chain_id
        })
        const unionClient = createUnionClient({
          account: cosmosOfflineSigner,
          transport: http(`${rpcUrl}`),
          chainId: sourceChain.chain_id as CosmosChainId,
          gasPrice: { amount: "0.025", denom: "ustars" } // TODO: don't hardcode
        })

        const transfer = await unionClient.transferAsset(transferArgs)
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
  } else if (sourceChain.rpc_type === "evm") {
    const connectorClient = await getConnectorClient(config)
    const selectedChain = evmChainFromChainId(sourceChain.chain_id)

    const unionClient = createUnionClient({
      account: connectorClient.account,
      chainId: sourceChain.chain_id as EvmChainId,
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
        const approve = await unionClient.approveErc20(transferArgs)

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
        const transfer = await unionClient.transferAsset(transferArgs)
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

    const transfer = $transferState.transferHash
    if (!transfer) {
      console.error("submitted invalid transfer. this should never happen")
      console.error("submitted invalid transfer. this should never happen. please contact the devs")
      goto(`/explorer/transfers/${$transferState.transferHash}`)
      return
    }

    // submittedTransfers.update(ts => {
    //   ts[$transferState.transferHash] = {
    //     _is_submitted_transfer: true,
    //     source_chain_id: transfer.sourceChain.chain_id,
    //     destination_chain_id: transfer.destinationChain.chain_id,
    //     packet_send_transaction_hash: $transferState.transferHash,
    //     sender: transfer.sender,
    //     transfer_day: toIsoString(new Date(Date.now())).split("T")[0],
    //     receiver: transfer.receiver,
    //     base_token: transfer.asset.metadata.denom,
    //     base_amount: parsedAmount
    //   }
    //   return ts
    // })
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

let stepperSteps = derived(transferState, $transferState => {
  if ($transferState.kind === "PRE_TRANSFER") return [] // don"t generate steps before transfer is ready
  const sourceChain = chains.find(c => c.chain_id === channel.source_chain_id)

  if (!sourceChain) {
    toast.error("no source chain found")
    return []
  }

  const destChain = chains.find(c => c.chain_id === channel.destination_chain_id)
  if (!destChain) {
    toast.error("no destination chain found")
    return []
  }

  if (sourceChain.rpc_type === "evm") {
    // TODO: Refactor this by implementing Ord for transferState
    return [
      // Do not uncomment
      stateToStatus(
        $transferState,
        "SWITCHING_TO_CHAIN",
        `Switch to ${sourceChain.display_name}`,
        `Switched to ${sourceChain.display_name}`,
        ts => ({
          status: "ERROR",
          title: `Error switching to ${sourceChain.display_name}`,
          description: `There was an issue switching to ${sourceChain.display_name} to your wallet. ${ts.warning}`
        }),
        () => ({
          status: "WARNING",
          title: `Could not automatically switch chain.`,
          description: `Please make sure your wallet is connected to  ${sourceChain.display_name}`
        }),
        () => ({
          status: "IN_PROGRESS",
          title: `Switching to ${sourceChain.display_name}`,
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
          description: `Waiting on ${sourceChain.display_name}`
        })
      ),
      stateToStatus(
        $transferState,
        "SIMULATING_TRANSFER",
        "Simulate transfer",
        "Simulated transfer",
        ts => ({
          status: "ERROR",
          title: `Error simulating transfer on ${sourceChain.display_name}`,
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
          description: `Waiting on ${sourceChain.display_name}`
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
          description: `Waiting on ${sourceChain.display_name}`
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
  if (sourceChain.rpc_type === "cosmos" || sourceChain.rpc_type === "aptos") {
    return [
      stateToStatus(
        $transferState,
        "SWITCHING_TO_CHAIN",
        `Switch to ${sourceChain.display_name}`,
        `Switched to ${sourceChain.display_name}`,
        ts => ({
          status: "ERROR",
          title: `Error switching to ${sourceChain.display_name}`,
          description: `There was an issue switching to ${sourceChain.display_name} to your wallet. ${ts.warning}`
        }),
        () => ({
          status: "WARNING",
          title: `Could not automatically switch chain.`,
          description: `Please make sure your wallet is connected to  ${sourceChain.display_name}`
        }),
        () => ({
          status: "IN_PROGRESS",
          title: `Switching to ${sourceChain.display_name}`,
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
    {#if $transferState.kind === "PRE_TRANSFER"}
    <div class="flex flex-col gap-6">
      <div>
        <h3 class="font-supermolot font-bold uppercase text-xl">Base Asset</h3>
        <Token amount={transferArgs.baseAmount} denom={transferArgs.baseToken} chainId={channel.source_chain_id} {chains}/>
      </div>
      <div>
        <h3 class="font-supermolot font-bold uppercase text-xl">Quote Asset</h3>
        <Token amount={transferArgs.quoteAmount} denom={transferArgs.quoteToken} chainId={channel.destination_chain_id} {chains}/>
      </div>
      <div>
        <h3 class="font-supermolot font-bold uppercase text-xl">Recipient</h3>
        <Address showChain showRaw address={transferArgs.receiver} {chains} chainId={channel.destination_chain_id}/>
      </div>
    </div>
    <div class="flex flex-1 flex-col justify-end items-center">

      <div class="flex gap-4 text-muted-foreground text-xs">{channel?.source_connection_id} | {channel?.source_channel_id} <ArrowRightIcon />{channel?.destination_connection_id} | {channel?.destination_channel_id}</div> 
    </div>

    <Button
            class="w-full mt-2"
            on:click={transfer}>Confirm Transfer
    </Button>
    {:else}
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
</div>

