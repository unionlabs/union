<script lang="ts">
import {
  createPfmMemo,
  type EvmChainId,
  createUnionClient,
  type CosmosChainId,
  evmChainFromChainId,
  bech32ToBech32Address
} from "@union/client"
import {
  http,
  custom,
  switchChain,
  getConnectorClient,
  waitForTransactionReceipt
} from "@wagmi/core"
import { onMount } from "svelte"
import { page } from "$app/stores"
import { toast } from "svelte-sonner"
import { goto } from "$app/navigation"
import Chevron from "./chevron.svelte"
import { cn } from "$lib/utilities/shadcn.ts"
import { userAddrEvm } from "$lib/wallet/evm"
import type { Step } from "$lib/stepper-types"
import { config } from "$lib/wallet/evm/config"
import ChainDialog from "./chain-dialog.svelte"
import ChainButton from "./chain-button.svelte"
import { toIsoString } from "$lib/utilities/date"
import AssetsDialog from "./assets-dialog.svelte"
import { userAddrCosmos } from "$lib/wallet/cosmos"
import { truncate } from "$lib/utilities/format.ts"
import Stepper from "$lib/components/stepper.svelte"
import { raise, sleep } from "$lib/utilities/index.ts"
import type { OfflineSigner } from "@leapwallet/types"
import { userBalancesQuery } from "$lib/queries/balance"
import * as Card from "$lib/components/ui/card/index.ts"
import type { Chain, UserAddresses } from "$lib/types.ts"
import { Input } from "$lib/components/ui/input/index.js"
import { userAddrOnChain } from "$lib/utilities/address.ts"
import { cosmosStore } from "$/lib/wallet/cosmos/config.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import { getSupportedAsset } from "$lib/utilities/helpers.ts"
import CardSectionHeading from "./card-section-heading.svelte"
import ArrowLeftRight from "virtual:icons/lucide/arrow-left-right"
import { parseUnits, formatUnits, type HttpTransport } from "viem"
import { getCosmosChainInfo } from "$lib/wallet/cosmos/chain-info.ts"
import { submittedTransfers } from "$lib/stores/submitted-transfers.ts"
import { type Writable, writable, derived, get, type Readable } from "svelte/store"
import { type TransferState, stepBefore, stepAfter } from "$lib/transfer/transfer.ts"

export let chains: Array<Chain>
let userAddress: Readable<UserAddresses> = derived(
  [userAddrCosmos, userAddrEvm],
  ([$userAddrCosmos, $userAddrEvm]) => ({
    evm: $userAddrEvm,
    cosmos: $userAddrCosmos
  })
)

$: userBalances = userBalancesQuery({ chains, userAddr: $userAddress, connected: true })

// CURRENT FORM STATE
let fromChainId = writable("")
let toChainId = writable("")
let assetSymbol = writable("")
let assetAddress = writable("")

let transferState: Writable<TransferState> = writable({ kind: "PRE_TRANSFER" })

let amount = ""
$: amountLargerThanZero = Number.parseFloat(amount) > 0

const amountRegex = /[^0-9.]|\.(?=\.)|(?<=\.\d+)\./g
$: amount = amount.replaceAll(amountRegex, "")

let balanceCoversAmount: boolean
$: if ($fromChain && $asset && amount) {
  try {
    const supported = getSupportedAsset($fromChain, $asset.address)
    const decimals = supported ? supported?.decimals : 0
    const inputAmount = parseUnits(amount.toString(), decimals)
    const balance = BigInt($asset.balance.toString())
    balanceCoversAmount = inputAmount <= balance
  } catch (error) {
    console.error("Error parsing amount or balance:", error)
  }
}

const REDIRECT_DELAY_MS = 5000

let dialogOpenToken = false
let dialogOpenToChain = false
let dialogOpenFromChain = false

let toChain = derived(
  toChainId,
  $toChainId => chains.find(chain => chain.chain_id === $toChainId) ?? null
)

let fromChain = derived(
  fromChainId,
  $fromChainId => chains.find(chain => chain.chain_id === $fromChainId) ?? null
)

let prevAsset: string
$: asset = derived(
  [fromChain, userBalances, assetAddress],
  ([$fromChain, $userBalances, $assetAddress]) => {
    if ($fromChain === null || $assetAddress === "") return null

    const chainIndex = chains.findIndex(c => c.chain_id === $fromChainId)
    const userBalance = $userBalances[chainIndex]
    if (!userBalance.isSuccess) {
      return null
    }
    let balance = userBalance.data.find(balance => balance.address === $assetAddress)
    if (!balance) {
      return null
    }
    if (prevAsset !== balance.address) amount = ""
    prevAsset = balance.address
    return balance
  }
)

let recipient = derived([toChain, userAddress], ([$toChain, $userAddress]) => {
  switch ($toChain?.rpc_type) {
    case "evm": {
      const evmAddr = $userAddress.evm
      if (evmAddr === null) return null
      return $userAddress.evm?.canonical
    }
    case "cosmos": {
      const cosmosAddr = $userAddress.cosmos
      if (cosmosAddr === null) return null
      return bech32ToBech32Address({
        address: cosmosAddr.canonical,
        toPrefix: $toChain.addr_prefix
      })
    }
    default:
      return null
  }
})

let ucs01Configuration = derived(
  [fromChain, toChainId, recipient],
  ([$fromChain, $toChainId, $recipient]) => {
    if ($fromChain === null || $toChainId === null || $recipient === null) return null

    let ucs1_configuration =
      $toChainId in $fromChain.ucs1_configurations
        ? $fromChain.ucs1_configurations[$toChainId]
        : null

    let pfmMemo: string | null = null
    let hopChainId: string | null = null

    if (ucs1_configuration !== null) {
      // non-pfm transfer
      return { ucs1_configuration, hopChainId, pfmMemo }
    }

    // try finding pfm path
    for (const chain of chains) {
      let [foundHopChainId, ucs1Config] =
        Object.entries(chain.ucs1_configurations).find(
          ([, config]) => config.forwards[$toChainId] !== undefined
        ) ?? []
      if (foundHopChainId !== undefined && ucs1Config !== undefined) {
        hopChainId = foundHopChainId
        ucs1_configuration = $fromChain.ucs1_configurations[hopChainId]
        let forwardConfig = ucs1_configuration.forwards[$toChainId]
        if (!$recipient) return

        const _pfmMemo = createPfmMemo({
          channel: forwardConfig.channel_id,
          port: forwardConfig.port_id,
          // // @ts-expect-error
          receiver: $toChain?.rpc_type === "evm" ? $recipient.slice(2) : $recipient
        })
        if (_pfmMemo.isErr()) throw _pfmMemo.error
        pfmMemo = _pfmMemo.value
        break
      }
    }

    if (pfmMemo === null || hopChainId === null || ucs1_configuration === null) {
      return null
    }

    return { ucs1_configuration, hopChainId, pfmMemo }
  }
)

let hopChain = derived(ucs01Configuration, $ucs01Configuration => {
  if (!$ucs01Configuration) return null
  if ($ucs01Configuration.hopChainId === null) return null

  return chains.find(c => c.chain_id === $ucs01Configuration.hopChainId) ?? null
})

const transfer = async () => {
  if (!$assetSymbol) return toast.error("Please select an asset")
  if (!$asset) return toast.error(`Error finding asset ${$assetSymbol}`)
  if (!$fromChainId) return toast.error("Please select a from chain")
  if (!$fromChain) return toast.error("can't find chain in config")
  if (!$toChain) return toast.error("can't find chain in config")
  if (!$toChainId) return toast.error("Please select a to chain")
  if (!amount) return toast.error("Please select an amount")
  if ($fromChain.rpc_type === "evm" && !$userAddress.evm)
    return toast.error("No evm wallet connected")
  if ($fromChain.rpc_type === "cosmos" && !$userAddress.cosmos)
    return toast.error("No cosmos wallet connected")
  if (!$recipient) return toast.error("Invalid recipient")
  // if (!$ucs01Configuration)
  //   return toast.error(
  //     `No UCS01 configuration for ${$fromChain.display_name} -> ${$toChain.display_name}`
  //   )

  let supported = getSupportedAsset($fromChain, $asset.address)
  let decimals = supported?.decimals ?? 0
  let parsedAmount = parseUnits(amount, decimals)

  // let { ucs1_configuration, pfmMemo, hopChainId } = $ucs01Configuration
  if ($fromChain.rpc_type === "cosmos") {
    const { connectedWallet, connectionStatus } = get(cosmosStore)
    if ($userAddrCosmos === null) return toast.error("No Cosmos user address found")

    if (connectionStatus !== "connected" || !connectedWallet) {
      transferState.set({
        kind: "SWITCHING_TO_CHAIN",
        warning: new Error("No wallet connected")
      })
      return
    }

    const wallet = window[connectedWallet as "keplr" | "leap"]

    if (!wallet) {
      transferState.set({
        kind: "SWITCHING_TO_CHAIN",
        warning: new Error(`${connectedWallet} wallet not found`)
      })
      return
    }

    // @ts-ignore
    transferState.set({ kind: "SWITCHING_TO_CHAIN" })

    const rpcUrl = $fromChain.rpcs.find(rpc => rpc.type === "rpc")?.url
    if (!rpcUrl) return toast.error(`no rpc available for ${$fromChain.display_name}`)

    if (stepBefore($transferState, "CONFIRMING_TRANSFER")) {
      const chainInfo = getCosmosChainInfo($fromChainId, connectedWallet)

      if (chainInfo === null) {
        transferState.set({
          kind: "SWITCHING_TO_CHAIN",
          warning: new Error("Failed to switch chain")
        })
        return
      }

      try {
        await wallet.experimentalSuggestChain(chainInfo)
        await wallet.enable([$fromChainId])
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
        const cosmosOfflineSigner = (
          $cosmosStore.connectedWallet === "keplr"
            ? window?.keplr?.getOfflineSigner($fromChainId, {
                disableBalanceCheck: false
              })
            : window.leap
              ? window.leap.getOfflineSigner($fromChainId, {
                  disableBalanceCheck: false
                })
              : undefined
        ) as OfflineSigner

        const unionClient = createUnionClient({
          account: cosmosOfflineSigner,
          transport: http(`https://${rpcUrl}`),
          chainId: $fromChain.chain_id as CosmosChainId,
          gasPrice: { amount: "0.0025", denom: $assetSymbol }
        })

        const transfer = await unionClient.transferAsset({
          autoApprove: true,
          amount: parsedAmount,
          recipient: $recipient,
          denomAddress: $assetAddress,
          destinationChainId: $toChain.chain_id
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
  } else if ($fromChain.rpc_type === "evm") {
    const connectorClient = await getConnectorClient(config)
    const selectedChain = evmChainFromChainId($fromChainId)

    const unionClient = createUnionClient({
      account: connectorClient.account,
      chainId: $fromChain.chain_id as EvmChainId,
      transport: custom(window.ethereum) as unknown as HttpTransport
    })

    if (!selectedChain) {
      toast.error("From chain not found or supported")
      return
    }

    if ($userAddrEvm === null) return toast.error("No Cosmos user address found")
    // if (pfmMemo === null && $userAddrCosmos === null)
    //   return toast.error("Destination is a Cosmos chain, but no Cosmos user address found")
    // if (connectorClient?.chain?.id !== selectedChain.id) {
    // await windowEthereumAddChain(selectedChain)
    // await windowEthereumSwitchChain(selectedChain.id)
    //   await sleep(1_500)
    // }

    // const ucs01address = ucs1_configuration.contract_address as Address

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
          recipient: $recipient,
          denomAddress: $assetAddress,
          destinationChainId: $toChain.chain_id
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

      // if (pfmMemo === null && $userAddrCosmos === null)
      //   return toast.error("Destination is a Cosmos chain, but no Cosmos user address found")

      if ($transferState.warning) {
        transferState.set({ kind: "CONFIRMING_TRANSFER", contractRequest: null })
        transfer()
        return
      }

      // ^ the user is continuing continuing after having seen the warning

      console.log("confirming transfers test")

      try {
        // console.log("contract request", contractRequest)
        // const simulationResult = await simulateContract(config, contractRequest)
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
          recipient: $recipient,
          denomAddress: $assetAddress,
          destinationChainId: $toChain.chain_id
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
        source_chain_id: $fromChainId,
        destination_chain_id: $toChainId,
        source_transaction_hash: $transferState.transferHash,
        hop_chain_id: $hopChain?.chain_id,
        sender: userAddrOnChain($userAddress, $fromChain),
        normalized_sender:
          $fromChain?.rpc_type === "cosmos"
            ? $userAddrCosmos?.normalized
            : $userAddrEvm?.normalized,
        transfer_day: toIsoString(new Date(Date.now())).split("T")[0],
        receiver: $recipient,
        assets: {
          [$assetSymbol]: {
            info: $fromChain?.assets?.find(d => d.denom === $assetSymbol) ?? null,
            amount: parsedAmount
          }
        },
        amount
      }
      return ts
    })
    goto(`/${$transferState.transferHash}`)
  }
}
onMount(() => {
  const source = $page.url.searchParams.get("source")
  const asset = $page.url.searchParams.get("asset")

  fromChainId.subscribe(_ => {
    assetSymbol.set("")
  })

  if (source) {
    fromChainId.set(source)
  }

  if (asset) {
    assetSymbol.set(asset)
  }

  userAddrCosmos.subscribe(address => {
    if (address === null) {
      if ($fromChain?.rpc_type === "cosmos") {
        fromChainId.set("")
      }
      if ($toChain?.rpc_type === "cosmos") {
        toChainId.set("")
      }
    }
  })

  userAddrEvm.subscribe(address => {
    if (address === null) {
      if ($fromChain?.rpc_type === "evm") {
        fromChainId.set("")
      }
      if ($toChain?.rpc_type === "evm") {
        toChainId.set("")
      }
    }
  })
})

$: sendableBalances = derived([fromChainId, userBalances], ([$fromChainId, $userBalances]) => {
  if (!$fromChainId) return
  const chainIndex = chains.findIndex(c => c.chain_id === $fromChainId)
  const cosmosBalance = $userBalances[chainIndex]
  if (!cosmosBalance?.isSuccess || cosmosBalance.data instanceof Error) {
    console.log("trying to send from cosmos but no balances fetched yet")
    return null as any
  }
  return cosmosBalance.data.map(balance => ({
    ...balance,
    balance: BigInt(balance.balance)
  })) as any
})

function swapChainsClick(_event: MouseEvent) {
  const [fromChain, toChain] = [$fromChainId, $toChainId]
  toChainId.set(fromChain)
  fromChainId.set(toChain)
}

$: buttonText =
  $asset && amount
    ? balanceCoversAmount
      ? "transfer"
      : "insufficient balance"
    : $asset && !amount
      ? "enter amount"
      : "select asset and enter amount"

let supportedAsset: any
$: if ($fromChain && $asset) supportedAsset = getSupportedAsset($fromChain, $asset.address)

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

let stepperSteps = derived([fromChain, transferState], ([$fromChain, $transferState]) => {
  if ($transferState.kind === "PRE_TRANSFER") return [] // don't generate steps before transfer is ready
  if ($fromChain?.rpc_type === "evm") {
    // TODO: Refactor this by implementing Ord for transferState
    return [
      // Do not uncomment
      stateToStatus(
        $transferState,
        "SWITCHING_TO_CHAIN",
        `Switch to ${$fromChain.display_name}`,
        `Switched to ${$fromChain.display_name}`,
        ts => ({
          status: "ERROR",
          title: `Error switching to ${$fromChain.display_name}`,
          description: `There was an issue switching to ${$fromChain.display_name} to your wallet. ${ts.warning}`
        }),
        () => ({
          status: "WARNING",
          title: `Could not automatically switch chain.`,
          description: `Please make sure your wallet is connected to  ${$fromChain.display_name}`
        }),
        () => ({
          status: "IN_PROGRESS",
          title: `Switching to ${$fromChain.display_name}`,
          description: `Click 'Approve' in wallet.`
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
          description: `Waiting on ${$fromChain.display_name}`
        })
      ),
      stateToStatus(
        $transferState,
        "SIMULATING_TRANSFER",
        "Simulate transfer",
        "Simulated transfer",
        ts => ({
          status: "ERROR",
          title: `Error simulating transfer on ${$fromChain.display_name}`,
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
          description: `Waiting on ${$fromChain.display_name}`
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
          description: `Click 'Confirm' in your wallet`
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
          description: `Waiting on ${$fromChain.display_name}`
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
  if ($fromChain?.rpc_type === "cosmos") {
    return [
      stateToStatus(
        $transferState,
        "SWITCHING_TO_CHAIN",
        `Switch to ${$fromChain.display_name}`,
        `Switched to ${$fromChain.display_name}`,
        ts => ({
          status: "ERROR",
          title: `Error switching to ${$fromChain.display_name}`,
          description: `There was an issue switching to ${$fromChain.display_name} to your wallet. ${ts.warning}`
        }),
        () => ({
          status: "WARNING",
          title: `Could not automatically switch chain.`,
          description: `Please make sure your wallet is connected to  ${$fromChain.display_name}`
        }),
        () => ({
          status: "IN_PROGRESS",
          title: `Switching to ${$fromChain.display_name}`,
          description: `Click 'Approve' in wallet.`
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
          description: `Click 'Approve' in your wallet`
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

let inputState: "locked" | "unlocked" = "locked"

let userInput = false
$: address = $recipient ?? ""

$: if (!userInput && $recipient !== address) {
  address = $recipient ?? ""
}

const handleInput = (event: Event) => {
  address = (event.target as HTMLInputElement).value
  userInput = true
}

const resetInput = () => {
  userInput = false
  address = $recipient ?? ""
}
</script>

<div
  class={cn(
    'size-full duration-1000 transition-colors dark:bg-muted',
    $transferState.kind !== 'PRE_TRANSFER' ? 'bg-black/60' : '',
  )}
></div>

<div class="cube-scene" id="scene">
  <div
    class={cn('cube ', $transferState.kind !== 'PRE_TRANSFER' ? 'cube--flipped' : 'no-transition')}
  >
    <div class="cube-right font-bold flex items-center justify-center text-xl font-supermolot">
      UNION TESTNET
    </div>
    <Card.Root class={cn($transferState.kind === 'PRE_TRANSFER' ? 'no-transition' : 'cube-front')}>
      <Card.Header>
        <Card.Title>Transfer</Card.Title>
      </Card.Header>
      <Card.Content class={cn('flex flex-col gap-4')}>
        <section>
          <CardSectionHeading>From</CardSectionHeading>
          <ChainButton bind:dialogOpen={dialogOpenFromChain} bind:selectedChainId={$fromChainId}>
            {$fromChain?.display_name ?? 'Select chain'}
          </ChainButton>
          <div class="flex flex-col items-center pt-4 -mb-6">
            <Button on:click={swapChainsClick} size="icon" variant="outline">
              <ArrowLeftRight class="size-5 dark:text-white rotate-90" />
            </Button>
          </div>
          <CardSectionHeading>To</CardSectionHeading>
          <ChainButton bind:dialogOpen={dialogOpenToChain} bind:selectedChainId={$toChainId}>
            {$toChain?.display_name ?? 'Select chain'}
          </ChainButton>
        </section>
        <section>
          <CardSectionHeading>Asset</CardSectionHeading>
          {#if $sendableBalances !== undefined && $fromChainId}
            {#if $sendableBalances === null}
              Failed to load sendable balances for <b>{$fromChain?.display_name}</b>.
            {:else if $sendableBalances && $sendableBalances.length === 0}
              You don't have sendable assets on <b>{$fromChain?.display_name}</b>. You can get some
              from <a class="underline font-bold" href="/faucet">the faucet</a>
            {:else}
              <Button
                class="w-full"
                variant="outline"
                on:click={() => (dialogOpenToken = !dialogOpenToken)}
              >
                <div class="flex-1 text-left font-bold text-md">
                  {truncate(
                    supportedAsset
                      ? supportedAsset.display_symbol
                      : $assetSymbol
                        ? $assetSymbol
                        : 'Select Asset',
                    12,
                  )}
                </div>

                <Chevron />
              </Button>
            {/if}
          {:else}
            Select a chain to send from.
          {/if}
          {#if $assetSymbol !== '' && $sendableBalances !== null && $asset?.address}
            <div class="mt-4 text-xs text-muted-foreground">
              <b>{truncate(supportedAsset ? supportedAsset?.display_symbol : $assetSymbol, 12)}</b>
              balance on
              <b>{$fromChain?.display_name}</b> is
              {formatUnits(BigInt($asset.balance), supportedAsset?.decimals ?? 0)}
            </div>
          {/if}
        </section>

        <section>
          <CardSectionHeading>Amount</CardSectionHeading>
          <Input
            autocapitalize="none"
            autocomplete="off"
            autocorrect="off"
            type="number"
            inputmode="decimal"
            bind:value={amount}
            class={cn(
              !balanceCoversAmount && amount ? 'border-red-500' : '',
              'focus:ring-0 focus-visible:ring-0 disabled:bg-black/30',
            )}
            disabled={!$asset}
            maxlength={64}
            minlength={1}
            pattern="^[0-9]*[.,]?[0-9]*$"
            placeholder="0.00"
            spellcheck="false"
          />
        </section>
        <section>
          <CardSectionHeading>Recipient</CardSectionHeading>
          <div class="flex items-start gap-2">
            <div class="w-full">
              <div class="relative w-full mb-2">
                <Input
                  autocapitalize="none"
                  autocomplete="off"
                  autocorrect="off"
                  bind:value={address}
                  class="disabled:bg-black/30"
                  disabled={inputState === 'locked'}
                  id="address"
                  on:input={handleInput}
                  placeholder="Select chain"
                  required={true}
                  spellcheck="false"
                  type="text"
                />
              </div>
              <div class="flex justify-between px-1">
                {#if userInput}
                  <button
                    type="button"
                    on:click={resetInput}
                    class="text-xs text-muted-foreground hover:text-primary transition"
                  >
                    Reset
                  </button>
                {/if}
              </div>
            </div>
          </div>
        </section>
      </Card.Content>
      <Card.Footer class="flex flex-col gap-4 items-start">
        <Button
          disabled={!amount ||
            !$asset ||
            !$toChainId ||
            !$recipient ||
            !$assetSymbol ||
            !$fromChainId ||
            !amountLargerThanZero ||
            // >= because need some sauce for gas
            !balanceCoversAmount}
          on:click={async event => {
            event.preventDefault()
            transferState.set({ kind: 'FLIPPING' })
            await sleep(1200)
            transfer()
          }}
          type="button"
        >
          {buttonText}
        </Button>
      </Card.Footer>
    </Card.Root>

    {#if $transferState.kind !== 'PRE_TRANSFER'}
      <Card.Root class={cn('cube-back p-6')}>
        {#if $fromChain}
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
      </Card.Root>
      <div class="cube-left font-bold flex items-center justify-center text-xl font-supermolot">
        UNION TESTNET
      </div>
    {/if}
  </div>
</div>

<ChainDialog
  bind:dialogOpen={dialogOpenFromChain}
  chains={chains.filter(c => c.enabled_staging)}
  kind="from"
  onChainSelect={newSelectedChain => {
    fromChainId.set(newSelectedChain)
  }}
  selectedChain={$fromChainId}
  userAddress={$userAddress}
/>

<ChainDialog
  bind:dialogOpen={dialogOpenToChain}
  chains={chains.filter(c => c.enabled_staging)}
  kind="to"
  onChainSelect={newSelectedChain => {
    toChainId.set(newSelectedChain)
  }}
  selectedChain={$toChainId}
  userAddress={$userAddress}
/>

{#if $sendableBalances && $fromChain}
  <AssetsDialog
    chain={$fromChain}
    assets={$sendableBalances}
    onAssetSelect={asset => {
      assetSymbol.set(asset.symbol)
      assetAddress.set(asset.address)
    }}
    bind:dialogOpen={dialogOpenToken}
  />
{/if}

<style global lang="postcss">
  .cube-scene {
    @apply absolute -my-6 py-6 z-20;
    top: calc(50% - (var(--height) / 2));
    --width: calc(min(500px, (100dvw - 32px)));
    --height: calc(min(740px, (100dvh - 144px)));
    --depth: 80px;
    --speed: 2s;
    width: var(--width);
    height: var(--height);
    perspective: 1000px;
  }

  .cube {
    @apply relative;
    width: var(--width);
    height: var(--height);
    transform-style: preserve-3d;
    transition: transform var(--speed);
    transform: translateZ(calc(var(--depth) * -0.5)) rotateY(0deg);
  }

  .cube--flipped {
    transform: translateZ(calc(var(--depth) * -0.5)) rotateY(180deg);
  }

  .cube-front,
  .cube-back {
    @apply absolute overflow-y-auto overflow-x-hidden;

    width: var(--width);
    height: var(--height);
  }

  .cube-left {
    @apply absolute bg-card border;
    width: var(--height);
    height: var(--depth);
    top: calc((var(--height) / 2) - (var(--depth) / 2));
    right: calc((var(--width) / 2) - (var(--height) / 2));
    transform: rotateZ(90deg) translateY(calc(var(--width) * 0.5)) rotateX(-90deg);
  }

  .cube-right {
    @apply absolute bg-card border;
    width: var(--height);
    height: var(--depth);
    top: calc((var(--height) / 2) - (var(--depth) / 2));
    left: calc((var(--width) / 2) - (var(--height) / 2));
    transform: rotateZ(-90deg) translateY(calc(var(--width) * 0.5)) rotateX(-90deg);
  }

  .cube-front {
    transform: translateZ(calc(var(--depth) * 0.5));
  }

  .cube-back {
    transform: translateZ(calc(var(--depth) * -0.5)) rotateY(180deg);
  }

  .no-transition {
    @apply overflow-y-auto overflow-x-hidden;
    width: var(--width);
    height: var(--height);
    transition: none !important;
  }
</style>
