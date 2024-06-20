<script lang="ts">
import { onMount } from "svelte"
import { toast } from "svelte-sonner"
import { sepolia } from "viem/chains"
import Chevron from "./chevron.svelte"
import { UnionClient } from "@union/client"
import { cn } from "$lib/utilities/shadcn.ts"
import { sleep } from "$lib/utilities/index.ts"
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
import type { Address } from "viem"
import { goto } from "$app/navigation"
import { ucs01abi } from "$lib/abi/ucs-01.ts"

import type { Chain, UserAddresses } from "$lib/types.ts"
import CardSectionHeading from "./card-section-heading.svelte"
import ArrowLeftRight from "virtual:icons/lucide/arrow-left-right"
import { erc20Abi, createWalletClient, createPublicClient, http, custom, defineChain } from "viem"

export let chains: Array<Chain>
export let userAddr: UserAddresses
let userBalances = userBalancesQuery({ chains, userAddr })

// CURRENT FORM STATE
let fromChainId = writable("union-testnet-8")
let toChainId = writable("11155111")
let assetSymbol = writable("")

type TransferStates =
  | "PRE_TRANSFER"
  | "ADDING_CHAIN"
  | "SWITCHING_TO_CHAIN"
  | "APPROVING_ASSET"
  | "SIMULATING_TRANSFER"
  | "CONFIRMING_TRANSFER"
  | "TRANSFERRING"

let transferState: Writable<TransferStates> = writable("PRE_TRANSFER")

let amount = ""
const amountRegex = /[^0-9.]|\.(?=\.)|(?<=\.\d+)\./g
$: {
  amount = amount.replaceAll(amountRegex, "")
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

  let { ucs1_configuration, pfmMemo, hopChainId } = $ucs01Configuration

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
    toast.info("Confirming transfer")

    let transferAssetsMessage: Parameters<UnionClient["transferAssets"]>[0]
    if (ucs1_configuration.contract_address === "ics20") {
      transferAssetsMessage = {
        kind: "ibc",
        messageTransfers: [
          {
            sourcePort: "transfer",
            sourceChannel: ucs1_configuration.channel_id,
            token: { denom: $assetSymbol, amount },
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
            funds: [{ denom: $assetSymbol, amount }]
          }
        ]
      }
    }

    const cosmosTransfer = await cosmosClient.transferAssets(transferAssetsMessage)
    transferState.set("TRANSFERRING")
    toast.info("Transferring assets")
    await sleep(REDIRECT_DELAY_MS)
    goto(`/explorer/transfers/${cosmosTransfer.transactionHash}`)
  } else if ($fromChain.rpc_type === "evm") {
    transferState.set("ADDING_CHAIN")

    const rpcUrls = $fromChain.rpcs.filter(c => c.type === "rpc").map(c => `https://${c.url}`)

    if (rpcUrls.length === 0) return toast.error(`No RPC url for ${$fromChain.display_name}`)

    const nativeCurrency = $fromChain.assets.filter(asset => asset.denom === "native").at(0)

    if (nativeCurrency === undefined)
      return toast.error(`No native currency for ${$fromChain.display_name}`)

    const chain = defineChain({
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
      transport: http()
    })

    const walletClient = createWalletClient({
      chain,
      // @ts-ignore
      transport: custom(window.ethereum)
    })

    toast.info(`Adding chain ${$fromChain.display_name} to your wallet`)
    await walletClient.addChain({ chain })

    transferState.set("SWITCHING_TO_CHAIN")
    toast.info(`Switching wallet to chain ${$fromChain.display_name}`)
    await walletClient.switchChain({ id: chain.id })

    const ucs01address = ucs1_configuration.contract_address as Address

    transferState.set("APPROVING_ASSET")
    toast.info("submitting approval")
    const approveContractSimulation = await walletClient.writeContract({
      account: userAddr.evm.canonical,
      abi: erc20Abi,
      address: $asset.address as Address,
      functionName: "approve",
      args: [ucs01address, BigInt(amount)]
    })

    toast.info("Submitting approval")

    transferState.set("SIMULATING_TRANSFER")
    toast.info("Simulating UCS01 contract call")
    const { request } = await publicClient.simulateContract({
      abi: ucs01abi,
      account: userAddr.evm.canonical,
      functionName: "send",
      address: ucs01address,
      args: [
        ucs1_configuration.channel_id,
        userAddr.cosmos.normalized_prefixed, // TODO: make dependent on target
        [{ denom: $asset.address.toLowerCase() as Address, amount: BigInt(amount) }],
        pfmMemo ?? "", // memo
        { revision_number: 9n, revision_height: BigInt(999_999_999) + 100n },
        0n
      ]
    })

    transferState.set("CONFIRMING_TRANSFER")
    toast.info("Submitting UCS01 contract call")
    const hash = await walletClient.writeContract(request)

    transferState.set("TRANSFERRING")
    toast.info("Transferring assets")
    await sleep(REDIRECT_DELAY_MS)
    goto(`/explorer/transfers/${hash}`)
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
    ? BigInt(amount) < BigInt($asset.balance)
      ? "transfer"
      : "insufficient balance"
    : "select asset and enter amount"
</script>


{#if $transferState === "PRE_TRANSFER"}
<Card.Root>
  <Card.Header>
    <Card.Title>Transfers</Card.Title>
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
        class="size-full"
        variant="outline"
        on:click={() => (dialogOpenToken = !dialogOpenToken)}
      >
        <div class="flex-1 text-left">{truncate($assetSymbol, 12)}</div>

        <Chevron />
      </Button>
    {/if}
    {#if $assetSymbol !== '' && $sendableBalances !== null}
      <div class="mt-4 text-xs text-muted-foreground">
        <b>{truncate($assetSymbol, 12)}</b> balance on <b>{$fromChain?.display_name}</b> is
        <b>{$sendableBalances.find(b => b.symbol === $assetSymbol)?.balance}</b>
      </div>
    {/if}
  </section>

  <section>
    <CardSectionHeading>Amount</CardSectionHeading>
    <Input
      minlength={1}
      maxlength={64}
      placeholder="0.00"
      autocorrect="off"
      autocomplete="off"
      spellcheck="false"
      bind:value={amount}
      autocapitalize="none"
      pattern="^[0-9]*[.,]?[0-9]*$"
    />
  </section>
  <section>
    <CardSectionHeading>Recipient</CardSectionHeading>
    <div class="text-muted-foreground font-mono">{$recipient}</div>
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
      BigInt(amount) >= BigInt($asset.balance)}
    on:click={async event => {
      event.preventDefault()
      transfer()
    }}
  >
    {buttonText}
  </Button>
  <div class="text-muted-foreground">
    Will transfer {#if amount}<b>{amount} {truncate($assetSymbol, 6)}</b>{/if} from <b>{$fromChain?.display_name}</b> to {#if $recipient}<span class="font-bold font-mono">{$recipient}</span>{/if} on <b>{$toChain?.display_name}</b><span>{#if $hopChain}&nbsp;by forwarding through <b class="m-0">{$hopChain.display_name.trim()}</b>{/if}</span>. 
  </div>
</Card.Footer>
</Card.Root>
{:else}
  <div class="text-muted-foreground">
    Transferring {#if amount}<b>{amount} {truncate($assetSymbol, 6)}</b>{/if} from <b>{$fromChain?.display_name}</b> to {#if $recipient}<span class="font-bold font-mono">{$recipient}</span>{/if} on <b>{$toChain?.display_name}</b><span>{#if $hopChain}&nbsp;by forwarding through <b class="m-0">{$hopChain.display_name.trim()}</b>{/if}</span>. 
  </div>
  <pre>{$transferState}</pre>
{/if}


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
    assets={$sendableBalances}
    onAssetSelect={newSelectedAsset => {
      assetSymbol.set(newSelectedAsset)
    }}
    bind:dialogOpen={dialogOpenToken}
  />
{/if}
