<script lang="ts">
import { onMount } from "svelte"
import { toast } from "svelte-sonner"
import Chevron from "./chevron.svelte"
import { UnionClient } from "@union/client"
import { cn } from "$lib/utilities/shadcn.ts"
import { raise, sleep } from "$lib/utilities/index.ts"
import type { OfflineSigner } from "@leapwallet/types"
import * as Card from "$lib/components/ui/card/index.ts"
import { Input } from "$lib/components/ui/input/index.js"
import { cosmosStore } from "$/lib/wallet/cosmos/config.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import ChainDialog from "./chain-dialog.svelte"
import ChainButton from "./chain-button.svelte"
import AssetsDialog from "./assets-dialog.svelte"
import { truncate } from "$lib/utilities/format.ts"
import { type Writable, writable, derived } from "svelte/store"
import { rawToBech32, userAddrOnChain } from "$lib/utilities/address.ts"
import { userBalancesQuery } from "$lib/queries/balance"
import { page } from "$app/stores"
import { goto } from "$app/navigation"
import { ucs01abi } from "$lib/abi/ucs-01.ts"
import { type Address, parseUnits } from "viem"
import Stepper from "$lib/components/stepper.svelte"
import { type TransferState, stepBefore, stepAfter } from "$lib/transfer/transfer.ts"
import type { Chain, UserAddresses } from "$lib/types.ts"
import CardSectionHeading from "./card-section-heading.svelte"
import ArrowLeftRight from "virtual:icons/lucide/arrow-left-right"
import { erc20Abi } from "viem"
import Precise from "$lib/components/precise.svelte"
import { getSupportedAsset } from "$lib/utilities/helpers.ts"
import { submittedTransfers } from "$lib/stores/submitted-transfers.ts"
import { toIsoString } from "$lib/utilities/date"
import { config } from "$lib/wallet/evm/config"
import {
  writeContract,
  simulateContract,
  waitForTransactionReceipt,
  switchChain
} from "@wagmi/core"

export let chains: Array<Chain>
export let userAddr: UserAddresses
export let connected: boolean
$: userBalances = userBalancesQuery({ chains, userAddr, connected })

// CURRENT FORM STATE
let fromChainId = writable("")
let toChainId = writable("")
let assetSymbol = writable("")

let transferState: Writable<TransferState> = writable({ kind: "PRE_TRANSFER" })

let amount = ""
$: amountLargerThanZero = Number.parseFloat(amount) > 0

const amountRegex = /[^0-9.]|\.(?=\.)|(?<=\.\d+)\./g
$: {
  amount = amount.replaceAll(amountRegex, "")
}

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

const REDIRECT_DELAY_MS = 2500

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

$: asset = derived(
  [assetSymbol, fromChain, userBalances],
  ([$assetSymbol, $fromChain, $userBalances]) => {
    if ($assetSymbol === "" || $fromChain === null) return null

    const chainIndex = chains.findIndex(c => c.chain_id === $fromChainId)
    const userBalance = $userBalances[chainIndex]
    if (!userBalance.isSuccess) {
      return null
    }
    let balance = userBalance.data.find(balance => balance.symbol === $assetSymbol)
    if (!balance) {
      return null
    }
    return balance
  }
)

let recipient = derived(toChain, $toChain => {
  switch ($toChain?.rpc_type) {
    case "evm":
      return userAddr.evm.canonical
    case "cosmos":
      return rawToBech32($toChain.addr_prefix, userAddr.cosmos.bytes)
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
          ([foundHopChainId, config]) => config.forward[$toChainId] !== undefined
        ) ?? []
      if (foundHopChainId !== undefined && ucs1Config !== undefined) {
        hopChainId = foundHopChainId
        ucs1_configuration = $fromChain.ucs1_configurations[hopChainId]
        let forwardConfig = ucs1_configuration.forward[$toChainId]
        pfmMemo = generatePfmMemo(forwardConfig.channel_id, forwardConfig.port, $recipient.slice(2))
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
  if ($ucs01Configuration === null) return null
  if ($ucs01Configuration.hopChainId === null) return null

  return chains.find(c => c.chain_id === $ucs01Configuration.hopChainId) ?? null
})

const generatePfmMemo = (channel: string, port: string, receiver: string): string => {
  return JSON.stringify({
    forward: {
      port,
      channel,
      receiver
    }
  })
}

const transfer = async () => {
  if (!$assetSymbol) return toast.error("Please select an asset")
  if (!$asset) return toast.error(`Error finding asset ${$assetSymbol}`)
  if (!$fromChainId) return toast.error("Please select a from chain")
  if (!$fromChain) return toast.error("can't find chain in config")
  if (!$toChain) return toast.error("can't find chain in config")
  if (!$toChainId) return toast.error("Please select a to chain")
  if (!amount) return toast.error("Please select an amount")
  if (!userAddr.evm) return toast.error("No evm wallet connected")
  if (!userAddr.cosmos) return toast.error("No cosmos wallet connected")
  if (!$recipient) return toast.error("Invalid recipient")
  if (!$ucs01Configuration)
    return toast.error(
      `No UCS01 configuration for ${$fromChain.display_name} -> ${$toChain.display_name}`
    )

  let supported = getSupportedAsset($fromChain, $asset.address)
  let decimals = supported?.decimals ?? 0
  let parsedAmount = parseUnits(amount, decimals)

  let { ucs1_configuration, pfmMemo, hopChainId } = $ucs01Configuration
  if ($fromChain.rpc_type === "cosmos") {
    // @ts-ignore
    transferState.set({ kind: "CONFIRMING_TRANSFER" })
    const rpcUrl = $fromChain.rpcs.find(rpc => rpc.type === "rpc")?.url

    if (!rpcUrl) return toast.error(`no rpc available for ${$fromChain.display_name}`)

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
        let cosmosClient = new UnionClient({
          cosmosOfflineSigner,
          evmSigner: undefined,
          bech32Prefix: $fromChain.addr_prefix,
          chainId: $fromChain.chain_id,
          gas: { denom: $assetSymbol, amount: "0.0025" },
          rpcUrl: `https://${rpcUrl}`
        })

        let transferAssetsMessage: Parameters<UnionClient["transferAssets"]>[0]
        if (ucs1_configuration.contract_address === "ics20") {
          transferAssetsMessage = {
            kind: "ibc",
            messageTransfers: [
              {
                sourcePort: "transfer",
                sourceChannel: ucs1_configuration.channel_id,
                token: { denom: $assetSymbol, amount: parsedAmount.toString() },
                sender: rawToBech32($fromChain.addr_prefix, userAddr.cosmos.bytes),
                receiver: $recipient,
                memo: pfmMemo ?? "",
                timeoutHeight: { revisionHeight: 888888888n, revisionNumber: 8n }
              }
            ]
          }
        } else {
          transferAssetsMessage = {
            kind: "cosmwasm",
            instructions: [
              {
                contractAddress: ucs1_configuration.contract_address,
                msg: {
                  transfer: {
                    channel: ucs1_configuration.channel_id,
                    receiver: $recipient?.slice(2),
                    memo: pfmMemo ?? ""
                  }
                },
                funds: [{ denom: $assetSymbol, amount: parsedAmount.toString() }]
              }
            ]
          }
        }

        const cosmosTransfer = await cosmosClient.transferAssets(transferAssetsMessage)
        transferState.set({ kind: "TRANSFERRING", transferHash: cosmosTransfer.transactionHash })
      } catch (error) {
        if (error instanceof Error) {
          // @ts-ignore
          transferState.set({ kind: "CONFIRMING_TRANSFER", error })
        }
        return
      }
    }
  } else if ($fromChain.rpc_type === "evm") {
    const ucs01address = ucs1_configuration.contract_address as Address

    if (window.ethereum === undefined) raise("no ethereum browser extension")

    if (stepBefore($transferState, "SWITCHING_TO_CHAIN")) {
      transferState.set({ kind: "ADDING_CHAIN" })
      // try {
      //   // await walletClient.addChain({ chain: viemChain })
      // } catch (error) {
      //   if (error instanceof Error) {
      //     transferState.set({ kind: "ADDING_CHAIN", error })
      //   }
      //   return
      // }
      transferState.set({ kind: "SWITCHING_TO_CHAIN" })
    }

    if ($transferState.kind === "SWITCHING_TO_CHAIN") {
      try {
        await switchChain(config, { chainId: 11155111 })
      } catch (error) {
        if (error instanceof Error) {
          transferState.set({ kind: "SWITCHING_TO_CHAIN", error })
        }
        return
      }
      transferState.set({ kind: "APPROVING_ASSET" })
    }

    if ($transferState.kind === "APPROVING_ASSET") {
      let hash: `0x${string}` | null = null

      try {
        hash = await writeContract(config, {
          account: userAddr.evm.canonical,
          abi: erc20Abi,
          address: $asset.address as Address,
          functionName: "approve",
          args: [ucs01address, parsedAmount]
        })
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
      try {
        const simulationResult = await simulateContract(config, {
          abi: ucs01abi,
          account: userAddr.evm.canonical,
          functionName: "send",
          address: ucs01address,
          args: [
            ucs1_configuration.channel_id,
            userAddr.cosmos.normalized_prefixed, // TODO: make dependent on target
            [{ denom: $asset.address.toLowerCase() as Address, amount: parsedAmount }],
            pfmMemo ?? "", // memo
            { revision_number: 9n, revision_height: BigInt(999_999_999) + 100n },
            0n
          ]
        })
        // @ts-ignore
        transferState.set({ kind: "CONFIRMING_TRANSFER", simulationResult })
      } catch (error) {
        if (error instanceof Error) {
          transferState.set({ kind: "SIMULATING_TRANSFER", error })
        }
        return
      }
    }

    if ($transferState.kind === "CONFIRMING_TRANSFER") {
      try {
        // @ts-ignore
        const transferHash = await writeContract(config, $transferState.simulationResult.request)
        transferState.set({ kind: "AWAITING_TRANSFER_RECEIPT", transferHash })
      } catch (error) {
        if (error instanceof Error) {
          transferState.set({
            kind: "CONFIRMING_TRANSFER",
            simulationResult: $transferState.simulationResult,
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
    submittedTransfers.update(ts => {
      // @ts-ignore
      ts[$transferState.transferHash] = {
        source_chain_id: $fromChainId,
        destination_chain_id: $toChainId,
        source_transaction_hash: $transferState.transferHash,
        hop_chain_id: $hopChain?.chain_id,
        sender: userAddrOnChain(userAddr, $fromChain),
        normalized_sender:
          $fromChain?.rpc_type === "cosmos"
            ? userAddr?.cosmos?.normalized
            : userAddr?.evm?.normalized,
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
    await sleep(REDIRECT_DELAY_MS);
    transferState.set({ kind: "TRANSFERRED", transferHash: $transferState.transferHash }); 
  }

  if ($transferState.kind === "TRANSFERRED") {
    await sleep(REDIRECT_DELAY_MS);
    goto(`/explorer/transfers/${$transferState.transferHash}`)
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
})

$: sendableBalances = derived([fromChainId, userBalances], ([$fromChainId, $userBalances]) => {
  if (!$fromChainId) return
  const chainIndex = chains.findIndex(c => c.chain_id === $fromChainId)
  const cosmosBalance = $userBalances[chainIndex]
  if (!cosmosBalance?.isSuccess || cosmosBalance.data instanceof Error) {
    console.log("trying to send from cosmos but no balances fetched yet")
    return null
  }
  return cosmosBalance.data.map(balance => ({ ...balance, balance: BigInt(balance.balance) }))
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
  progressFormatter: (ts: Extract<TransferState, { kind: K }>) => unknown
) =>
  stepBefore(state, kind)
    ? { status: "PENDING", title: pendingTitle }
    : stepAfter(state, kind)
      ? { status: "COMPLETED", title: completedTitle }
      : // @ts-ignore
        state.error !== undefined
        ? errorFormatter(state as Extract<TransferState, { kind: K }>)
        : progressFormatter(state as Extract<TransferState, { kind: K }>)

let stepperSteps = derived([fromChain, transferState], ([$fromChain, $transferState]) => {
  if ($fromChain?.rpc_type === "evm") {
    // TODO: Refactor this by implementing Ord for transferState
    return [
      // stateToStatus(
      //   $transferState,
      //   "ADDING_CHAIN",
      //   `Add ${$fromChain.display_name}`,
      //   `Added ${$fromChain.display_name}`,
      //   ts => ({
      //     status: "ERROR",
      //     title: `Error adding ${$fromChain.display_name}`,
      //     description: `There was an issue adding ${$fromChain.display_name} to your wallet. ${ts.error}`
      //   }),
      //   () => ({
      //     status: "IN_PROGRESS",
      //     title: `Adding ${$fromChain.display_name}`,
      //     description: `Click 'Approve' in wallet.`
      //   })
      // ),
      stateToStatus(
        $transferState,
        "SWITCHING_TO_CHAIN",
        `Switch to ${$fromChain.display_name}`,
        `Switched to ${$fromChain.display_name}`,
        ts => ({
          status: "ERROR",
          title: `Error switching to ${$fromChain.display_name}`,
          description: `There was an issue switching to ${$fromChain.display_name} to your wallet. ${ts.error}`
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
          description: `${ts.error}`
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
        () => ({
          status: "IN_PROGRESS",
          title: "Transferring assets",
          description: `Successfully initiated transfer`
        })
      )
    ]
  }
  if ($fromChain?.rpc_type === "cosmos") {
    return [
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
        () => ({
          status: "IN_PROGRESS",
          title: "Transferring assets",
          description: `Successfully initiated transfer`
        })
      )
    ]
  }
  raise("trying to make stepper for unsupported chain")
})

let inputState: "locked" | "unlocked" = "locked"
const onLockClick = () => (inputState = inputState === "locked" ? "unlocked" : "locked")

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
  class={cn("size-full duration-1000	 transition-colors bg-background", $transferState.kind !== "PRE_TRANSFER" ? "bg-black/60" : "")}></div>
<div class="cube-scene">
  <div class={cn("cube", $transferState.kind !== "PRE_TRANSFER" ? "cube--flipped" : "")}>
    <Card.Root class="cube-front">
      <Card.Header>
        <Card.Title>Transfer</Card.Title>
      </Card.Header>
      <Card.Content class={cn('flex flex-col gap-4')}>
        <section>
          <CardSectionHeading>From</CardSectionHeading>
          <ChainButton bind:dialogOpen={dialogOpenFromChain} bind:selectedChainId={$fromChainId}>
            {$fromChain?.display_name ?? "Select chain"}
          </ChainButton>

          <div class="flex flex-col items-center pt-4 -mb-6">
            <Button on:click={swapChainsClick} size="icon" variant="outline">
              <ArrowLeftRight class="size-5 dark:text-white rotate-90"/>
            </Button>
          </div>

          <CardSectionHeading>To</CardSectionHeading>
          <ChainButton bind:dialogOpen={dialogOpenToChain} bind:selectedChainId={$toChainId}>
            {$toChain?.display_name ?? "Select chain"}
          </ChainButton>
        </section>
        <section>
          <CardSectionHeading>Asset</CardSectionHeading>
          {#if $sendableBalances}
            {#if $sendableBalances === null}
              Failed to load sendable balances for <b>{$fromChain?.display_name}</b>.
            {:else if $sendableBalances && $sendableBalances.length === 0}
              You don't have sendable balances on <b>{$fromChain?.display_name}</b>.
            {:else}
              <Button
                class="w-full"
                variant="outline"
                on:click={() => (dialogOpenToken = !dialogOpenToken)}
              >
                <div
                  class="flex-1 text-left font-bold text-md">{truncate(supportedAsset ? supportedAsset.display_symbol : $assetSymbol ? $assetSymbol : 'Select Asset', 12)}</div>

                <Chevron/>
              </Button>
            {/if}
          {:else}
            Select a chain to send from.
          {/if}
          {#if $assetSymbol !== '' && $sendableBalances !== null && $asset?.address}
            <div class="mt-4 text-xs text-muted-foreground">
              <b>{truncate(supportedAsset ? supportedAsset.display_symbol : $assetSymbol, 12)}</b> balance on
              <b>{$fromChain?.display_name}</b> is
              <Precise chain={$fromChain} asset={$asset} showToolTip/>
              <!--        <b>{$sendableBalances.find(b => b.symbol === $assetSymbol)?.balance}</b>-->
            </div>
          {/if}
        </section>

        <section>
          <CardSectionHeading>Amount</CardSectionHeading>
          <Input
            autocapitalize="none"
            autocomplete="off"
            autocorrect="off"
            bind:value={amount}
            class={cn(!balanceCoversAmount && amount ? 'border-red-500' : '')}
            disabled={
          !$asset
          }
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
                  class="disabled:opacity-100 disabled:bg-black/20"
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
            <!--            <Button-->
            <!--              aria-label="Toggle address lock"-->
            <!--              class="px-3"-->
            <!--              on:click={onLockClick}-->
            <!--              variant="ghost"-->
            <!--            >-->
            <!--              {#if inputState === 'locked'}-->
            <!--                <LockLockedIcon class="size-4.5"/>-->
            <!--              {:else}-->
            <!--                <LockOpenIcon class="size-4.5"/>-->
            <!--              {/if}-->
            <!--            </Button>-->
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
          !balanceCoversAmount
          }
          on:click={async event => {
          event.preventDefault()
          transferState.set({ kind: "FLIPPING" })
          await sleep(1200)
          transfer()
        }}
          type="button"
        >
          {buttonText}
        </Button>
      </Card.Footer>
    </Card.Root>

    <Card.Root class="cube-back p-6">
      {#if $fromChain}
        <Stepper steps={stepperSteps} onRetry={() => {
        transferState.update(ts => {
          // @ts-ignore
          ts.error = undefined; 
          return ts
        });

        transfer()
      }}/>
      {/if}
    </Card.Root>
    <div class="cube-left font-bold flex items-center justify-center text-xl font-supermolot">UNION TESTNET</div>
  </div>
</div>


<ChainDialog
  bind:dialogOpen={dialogOpenFromChain}
  {chains}
  connected={connected}
  kind="from"
  onChainSelect={newSelectedChain => {
    fromChainId.set(newSelectedChain)
  }}
  selectedChain={$fromChainId}
  userAddr={userAddr}
/>

<ChainDialog
  bind:dialogOpen={dialogOpenToChain}
  {chains}
  connected={connected}
  kind="to"
  onChainSelect={newSelectedChain => {
    toChainId.set(newSelectedChain)
  }}
  selectedChain={$toChainId}
  userAddr={userAddr}
/>

{#if $sendableBalances !== null}
  <AssetsDialog
    chain={$fromChain}
    assets={$sendableBalances}
    onAssetSelect={newSelectedAsset => {
      assetSymbol.set(newSelectedAsset)
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

    .cube-front, .cube-back {
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

    .cube-front {
        transform: translateZ(calc(var(--depth) * 0.5));
    }

    .cube-back {
        transform: translateZ(calc(var(--depth) * -0.5)) rotateY(180deg);
    }

</style>
