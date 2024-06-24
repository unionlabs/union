<script lang="ts">
import { onMount } from "svelte"
import { toast } from "svelte-sonner"
import { sepolia, berachainTestnetbArtio } from "viem/chains"
import Chevron from "./chevron.svelte"
import { UnionClient } from "@union/client"
import { cn } from "$lib/utilities/shadcn.ts"
import { raise, sleep } from "$lib/utilities/index.ts"
import { getWalletClient } from "@wagmi/core"
import { type Writable, writable, derived } from "svelte/store"
import { evmAccount } from "$lib/wallet/evm/stores.ts"
import type { OfflineSigner } from "@leapwallet/types"
import * as Card from "$lib/components/ui/card/index.ts"
import { Input } from "$lib/components/ui/input/index.js"
import { cosmosStore } from "$/lib/wallet/cosmos/config.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import ChainDialog from "./chain-dialog.svelte"
import ChainButton from "./chain-button.svelte"
import AssetsDialog from "./assets-dialog.svelte"
import { config } from "$lib/wallet/evm/config.ts"
import { truncate } from "$lib/utilities/format.ts"
import { rawToBech32 } from "$lib/utilities/address.ts"
import { userBalancesQuery } from "$lib/queries/balance"
import { page } from "$app/stores"
import { type Address, parseUnits } from "viem"
import { goto } from "$app/navigation"
import { ucs01abi } from "$lib/abi/ucs-01.ts"
import Stepper from "$lib/components/stepper.svelte"

import type { Chain, UserAddresses } from "$lib/types.ts"
import CardSectionHeading from "./card-section-heading.svelte"
import ArrowLeftRight from "virtual:icons/lucide/arrow-left-right"
import {
  erc20Abi,
  createWalletClient,
  createPublicClient,
  http,
  custom,
  defineChain,
  publicActions,
  fallback
} from "viem"
import Precise from "$lib/components/precise.svelte"
import { getSupportedAsset } from "$lib/utilities/helpers.ts"

export let chains: Array<Chain>
export let userAddr: UserAddresses
let userBalances = userBalancesQuery({ chains, userAddr })

// CURRENT FORM STATE
let fromChainId = writable("union-testnet-8")
let toChainId = writable("11155111")
let assetSymbol = writable("")

type TransferStates =
  | "PRE_TRANSFER"
  | "FLIPPING"
  | "ADDING_CHAIN"
  | "SWITCHING_TO_CHAIN"
  | "APPROVING_ASSET"
  | "AWAITING_APPROVAL_RECEIPT"
  | "SIMULATING_TRANSFER"
  | "CONFIRMING_TRANSFER"
  | "AWAITING_TRANSFER_RECEIPT"
  | "TRANSFERRING"

let transferState: Writable<TransferStates> = writable("PRE_TRANSFER")

let amount = ""
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

let asset = derived(
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
  if (!$recipient) return toast.error("Invalid recipient")
  if (!$ucs01Configuration)
    return toast.error(
      `No UCS01 configuration for ${$fromChain.display_name} -> ${$toChain.display_name}`
    )

  let formattedAmount = parseUnits(amount, $fromChain.assets[0].decimals)

  let { ucs1_configuration, pfmMemo, hopChainId } = $ucs01Configuration
  transferState.set("FLIPPING")
  await sleep(1200)

  if ($fromChain.rpc_type === "cosmos") {
    const rpcUrl = $fromChain.rpcs.find(rpc => rpc.type === "rpc")?.url

    if (!rpcUrl) return toast.error(`no rpc available for ${$fromChain.display_name}`)

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

    transferState.set("CONFIRMING_TRANSFER")

    let transferAssetsMessage: Parameters<UnionClient["transferAssets"]>[0]
    if (ucs1_configuration.contract_address === "ics20") {
      transferAssetsMessage = {
        kind: "ibc",
        messageTransfers: [
          {
            sourcePort: "transfer",
            sourceChannel: ucs1_configuration.channel_id,
            token: { denom: $assetSymbol, amount: formattedAmount.toString() },
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
            funds: [{ denom: $assetSymbol, amount: formattedAmount.toString() }]
          }
        ]
      }
    }

    const cosmosTransfer = await cosmosClient.transferAssets(transferAssetsMessage)
    transferState.set("TRANSFERRING")
    await sleep(REDIRECT_DELAY_MS)
    goto(`/explorer/transfers/${cosmosTransfer.transactionHash}`)
  } else if ($fromChain.rpc_type === "evm") {
    transferState.set("ADDING_CHAIN")

    const rpcUrls = $fromChain.rpcs.filter(c => c.type === "rpc").map(c => `https://${c.url}`)

    if (rpcUrls.length === 0) return toast.error(`No RPC url for ${$fromChain.display_name}`)

    const nativeCurrency = $fromChain.assets.filter(asset => asset.denom === "native").at(0)

    if (nativeCurrency === undefined)
      return toast.error(`No native currency for ${$fromChain.display_name}`)

    const chain =
      $fromChainId === "11155111"
        ? sepolia
        : $fromChainId === "80084"
          ? berachainTestnetbArtio
          : defineChain({
              name: $fromChain.display_name,
              nativeCurrency: {
                name: nativeCurrency.display_name ?? nativeCurrency.display_symbol,
                /** 2-6 characters long */
                symbol: nativeCurrency.display_symbol,
                decimals: nativeCurrency.decimals
              },
              id: Number($fromChainId),
              rpcUrls: {
                default: {
                  http: rpcUrls
                }
              },
              testnet: $fromChain.testnet
            })

    const publicClient = createPublicClient({
      chain,
      transport: fallback(rpcUrls.map(url => http(url)))
    })

    const walletClient = createWalletClient({
      chain,
      // @ts-ignore
      transport: custom(window.ethereum)
    })

    await walletClient.addChain({ chain })

    transferState.set("SWITCHING_TO_CHAIN")
    await walletClient.switchChain({ id: chain.id })

    const ucs01address = ucs1_configuration.contract_address as Address

    transferState.set("APPROVING_ASSET")
    const approveContractSimulation = await walletClient.writeContract({
      account: userAddr.evm.canonical,
      abi: erc20Abi,
      address: $asset.address as Address,
      functionName: "approve",
      args: [ucs01address, formattedAmount]
    })

    transferState.set("AWAITING_APPROVAL_RECEIPT")
    const approvalReceipt = await publicClient.waitForTransactionReceipt({
      hash: approveContractSimulation
    })

    transferState.set("SIMULATING_TRANSFER")
    const simulationResult = await publicClient.simulateContract({
      abi: ucs01abi,
      account: userAddr.evm.canonical,
      functionName: "send",
      address: ucs01address,
      args: [
        ucs1_configuration.channel_id,
        userAddr.cosmos.normalized_prefixed, // TODO: make dependent on target
        [{ denom: $asset.address.toLowerCase() as Address, amount: formattedAmount }],
        pfmMemo ?? "", // memo
        { revision_number: 9n, revision_height: BigInt(999_999_999) + 100n },
        0n
      ]
    })
    console.log("simulation result", simulationResult)

    transferState.set("CONFIRMING_TRANSFER")
    const transferHash = await walletClient.writeContract(simulationResult.request)

    transferState.set("AWAITING_TRANSFER_RECEIPT")
    const transferReceipt = await publicClient.waitForTransactionReceipt({
      hash: transferHash
    })

    transferState.set("TRANSFERRING")
    await sleep(REDIRECT_DELAY_MS)
    goto(`/explorer/transfers/${transferHash}`)
  } else {
    console.error("invalid rpc type")
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

let sendableBalances = derived([fromChainId, userBalances], ([$fromChainId, $userBalances]) => {
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

let stepperSteps = derived([fromChain, transferState], ([$fromChain, $transferState]) => {
  if ($fromChain?.rpc_type === "evm") {
    // TODO: Refactor this by implementing Ord for transferState
    return [
      {
        status:
          $transferState === "PRE_TRANSFER" || $transferState === "FLIPPING"
            ? "PENDING"
            : $transferState === "ADDING_CHAIN"
              ? "IN_PROGRESS"
              : "COMPLETED",
        title: `Adding ${$fromChain.display_name}`,
        description: "Click 'Add Chain' in your wallet."
      },
      {
        status:
          $transferState === "PRE_TRANSFER" ||
          $transferState === "FLIPPING" ||
          $transferState === "ADDING_CHAIN"
            ? "PENDING"
            : $transferState === "SWITCHING_TO_CHAIN"
              ? "IN_PROGRESS"
              : "COMPLETED",
        title: `Switching to ${$fromChain.display_name}`,
        description: "Click 'Switch to Chain' in your wallet."
      },
      {
        status:
          $transferState === "PRE_TRANSFER" ||
          $transferState === "FLIPPING" ||
          $transferState === "ADDING_CHAIN" ||
          $transferState === "SWITCHING_TO_CHAIN"
            ? "PENDING"
            : $transferState === "APPROVING_ASSET"
              ? "IN_PROGRESS"
              : "COMPLETED",
        title: `Approving ERC20`,
        description: "Click 'Next' and 'Approve' in wallet."
      },
      {
        status:
          $transferState === "PRE_TRANSFER" ||
          $transferState === "FLIPPING" ||
          $transferState === "ADDING_CHAIN" ||
          $transferState === "SWITCHING_TO_CHAIN" ||
          $transferState === "APPROVING_ASSET"
            ? "PENDING"
            : $transferState === "AWAITING_APPROVAL_RECEIPT"
              ? "IN_PROGRESS"
              : "COMPLETED",
        title: `Awaiting approval receipt`,
        description: `Waiting on ${$fromChain.display_name}`
      },
      {
        status:
          $transferState === "PRE_TRANSFER" ||
          $transferState === "FLIPPING" ||
          $transferState === "ADDING_CHAIN" ||
          $transferState === "SWITCHING_TO_CHAIN" ||
          $transferState === "APPROVING_ASSET" ||
          $transferState === "AWAITING_APPROVAL_RECEIPT"
            ? "PENDING"
            : $transferState === "SIMULATING_TRANSFER"
              ? "IN_PROGRESS"
              : "COMPLETED",
        title: `Simulating transfer`,
        description: `Waiting on ${$fromChain.display_name}`
      },
      {
        status:
          $transferState === "PRE_TRANSFER" ||
          $transferState === "FLIPPING" ||
          $transferState === "ADDING_CHAIN" ||
          $transferState === "SWITCHING_TO_CHAIN" ||
          $transferState === "APPROVING_ASSET" ||
          $transferState === "AWAITING_APPROVAL_RECEIPT" ||
          $transferState === "SIMULATING_TRANSFER"
            ? "PENDING"
            : $transferState === "CONFIRMING_TRANSFER"
              ? "IN_PROGRESS"
              : "COMPLETED",
        title: `Confirm your transfer`,
        description: `Click 'Confirm' in your wallet`
      },
      {
        status:
          $transferState === "PRE_TRANSFER" ||
          $transferState === "FLIPPING" ||
          $transferState === "ADDING_CHAIN" ||
          $transferState === "SWITCHING_TO_CHAIN" ||
          $transferState === "APPROVING_ASSET" ||
          $transferState === "AWAITING_APPROVAL_RECEIPT" ||
          $transferState === "SIMULATING_TRANSFER" ||
          $transferState === "CONFIRMING_TRANSFER"
            ? "PENDING"
            : $transferState === "AWAITING_TRANSFER_RECEIPT"
              ? "IN_PROGRESS"
              : "COMPLETED",
        title: `Awaiting transfer receipt`,
        description: `Waiting on ${$fromChain.display_name}`
      },
      {
        status:
          $transferState === "PRE_TRANSFER" ||
          $transferState === "FLIPPING" ||
          $transferState === "ADDING_CHAIN" ||
          $transferState === "SWITCHING_TO_CHAIN" ||
          $transferState === "APPROVING_ASSET" ||
          $transferState === "AWAITING_APPROVAL_RECEIPT" ||
          $transferState === "SIMULATING_TRANSFER" ||
          $transferState === "CONFIRMING_TRANSFER" ||
          $transferState === "AWAITING_TRANSFER_RECEIPT"
            ? "PENDING"
            : $transferState === "TRANSFERRING"
              ? "COMPLETED"
              : "ERROR",
        title: `Transferring your assets`,
        description: `Successfully initiated transfer`
      }
    ]
  }
  if ($fromChain?.rpc_type === "cosmos") {
    return [
      {
        status:
          $transferState === "PRE_TRANSFER" || $transferState === "FLIPPING"
            ? "PENDING"
            : $transferState === "CONFIRMING_TRANSFER"
              ? "IN_PROGRESS"
              : "COMPLETED",
        title: `Approving transfer`,
        description: "Click 'Approve' in your wallet."
      },
      {
        status:
          $transferState === "PRE_TRANSFER" ||
          $transferState === "FLIPPING" ||
          $transferState === "CONFIRMING_TRANSFER"
            ? "PENDING"
            : $transferState === "TRANSFERRING"
              ? "COMPLETED"
              : "ERROR",
        title: `Transferring your assets`,
        description: "Successfully initiated transfer"
      }
    ]
  }
  raise("trying to make stepper for unsupported chain")
})
</script>


<div class={cn("size-full duration-1000	 transition-colors bg-background", $transferState !== "PRE_TRANSFER" ? "bg-black/60" : "")}></div>
<div class="cube-scene">
  <div class={cn("cube", $transferState !== "PRE_TRANSFER" ? "cube--flipped" : "")}>
    <Card.Root class="cube-front">
      <Card.Header>
        <Card.Title>Transfer</Card.Title>
      </Card.Header>
    <Card.Content class={cn('flex flex-col gap-4')}>
      <section>
        <CardSectionHeading>From</CardSectionHeading>
        <ChainButton bind:selectedChainId={$fromChainId} bind:dialogOpen={dialogOpenFromChain}>
          {$fromChain?.display_name}
        </ChainButton>

        <div class="flex flex-col items-center pt-4 -mb-6">
          <Button size="icon" variant="outline" on:click={swapChainsClick}>
            <ArrowLeftRight class="size-5 dark:text-white rotate-90" />
          </Button>
        </div>

        <CardSectionHeading>To</CardSectionHeading>
        <ChainButton bind:selectedChainId={$toChainId} bind:dialogOpen={dialogOpenToChain}>
          {$toChain?.display_name}
        </ChainButton>
      </section>
      <section>
        <CardSectionHeading>Asset</CardSectionHeading>
        {#if $sendableBalances === null}
          Failed to load sendable balances for <b>{$fromChain?.display_name}</b>.
        {:else if $sendableBalances.length === 0}
          You don't have sendable balances on <b>{$fromChain?.display_name}</b>.
        {:else}
          <Button
            class="w-full"
            variant="outline"
            on:click={() => (dialogOpenToken = !dialogOpenToken)}
          >
            <div class="flex-1 text-left">{truncate(supportedAsset ? supportedAsset.display_symbol : $assetSymbol, 12)}</div>

            <Chevron />
          </Button>
        {/if}
        {#if $assetSymbol !== '' && $sendableBalances !== null}
          <div class="mt-4 text-xs text-muted-foreground">
            <b>{truncate(supportedAsset ? supportedAsset.display_symbol : $assetSymbol, 12)}</b> balance on <b>{$fromChain?.display_name}</b> is
            <Precise chain={$fromChain} asset={$asset} showToolTip/>
<!--        <b>{$sendableBalances.find(b => b.symbol === $assetSymbol)?.balance}</b>-->
          </div>
        {/if}
      </section>

      <section>
        <CardSectionHeading>Amount</CardSectionHeading>
        <Input
          disabled={
          !$asset
          }
          minlength={1}
          maxlength={64}
          placeholder="0.00"
          autocorrect="off"
          autocomplete="off"
          spellcheck="false"
          bind:value={amount}
          autocapitalize="none"
          pattern="^[0-9]*[.,]?[0-9]*$"
          class={cn(!balanceCoversAmount && amount ? 'border-red-500' : '')}
        />
      </section>
      <section>
        <CardSectionHeading>Recipient</CardSectionHeading>
        <div class="text-muted-foreground font-mono text-xs sm:text-base">{$recipient}</div>
      </section>
    </Card.Content>
    <Card.Footer class="flex flex-col gap-4 items-start">
      <Button
        type="button"
        disabled={!amount ||
          !$asset ||
          !$toChainId ||
          !$recipient ||
          !$assetSymbol ||
          !$fromChainId ||
          // >= because need some sauce for gas
          !balanceCoversAmount
          }
        on:click={async event => {
          event.preventDefault()
          transfer()
        }}
      >
        {buttonText}
      </Button>
    </Card.Footer>
    </Card.Root>

    <Card.Root class="cube-back p-6">
      <Stepper steps={$stepperSteps}/>
    </Card.Root>
    <div class="cube-left font-bold flex items-center justify-center text-xl font-supermolot">UNION UNION UNION UNION UNION UNION UNION UNION</div>
  </div>
</div>





<ChainDialog
  kind="from"
  {chains}
  selectedChain={$fromChainId}
  onChainSelect={newSelectedChain => {
    fromChainId.set(newSelectedChain)
  }}
  bind:dialogOpen={dialogOpenFromChain}
/>

<ChainDialog
  kind="to"
  {chains}
  selectedChain={$toChainId}
  onChainSelect={newSelectedChain => {
    toChainId.set(newSelectedChain)
  }}
  bind:dialogOpen={dialogOpenToChain}
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
    --height: calc(min(700px, (100dvh - 164px)));
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
    @apply absolute overflow-auto;

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
    transform: translateZ(calc(var(--depth) * -0.5)) rotateY(180deg) ;
  }

</style>
