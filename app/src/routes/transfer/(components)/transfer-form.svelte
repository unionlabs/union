<script lang="ts">
import { onMount } from "svelte"
import { writable } from "svelte/store"
import { toast } from "svelte-sonner"
import { sepolia } from "viem/chains"
import { UnionClient } from "@union/client"
import { cn } from "$lib/utilities/shadcn.ts"
import { getWalletClient } from "@wagmi/core"
import Chevron from "./chevron.svelte"
import { createWalletClient, isAddress } from "viem"
import { evmAccount } from "$lib/wallet/evm/stores.ts"
import type { OfflineSigner } from "@leapwallet/types"
import * as Card from "$lib/components/ui/card/index.ts"
import { Input } from "$lib/components/ui/input/index.js"
import { cosmosStore } from "$/lib/wallet/cosmos/config.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import ChainDialog from "./chain-dialog.svelte"
import ChainButton from "./chain-button.svelte"
import AssetsDialog from "./assets-dialog.svelte"
import ArrowLeftRight from "virtual:icons/lucide/arrow-left-right"
import CardSectionHeading from "./card-section-heading.svelte"
import { config } from "$lib/wallet/evm/config.ts"
import { cosmosBalancesQuery, evmBalancesQuery } from "$lib/queries/balance"
import { derived } from "svelte/store"
import { truncate } from "$lib/utilities/format.ts"
import { rawToBech32 } from "$lib/utilities/address.ts"
import type { Chain, UserAddresses } from "$lib/types.ts"

export let chains: Array<Chain>
export let userAddr: UserAddresses

let cosmosChains = chains.filter(c => c.rpc_type === "cosmos")

// CURRENT FORM STATE
let fromChainId = writable("union-testnet-8")
let toChainId = writable("11155111")
let asset = writable("")

let amount = ""
const amountRegex = /[^0-9.]|\.(?=\.)|(?<=\.\d+)\./g
$: {
  amount = amount.replaceAll(amountRegex, "")
}

let dialogOpenToken = false
let dialogOpenToChain = false
let dialogOpenFromChain = false

let evmBalances = evmBalancesQuery({
  chainId: "11155111",
  address: userAddr.evm.canonical,
  tokenSpecification: "erc20"
})

let cosmosBalances = cosmosBalancesQuery({
  chains: cosmosChains,
  address: userAddr.cosmos.bytes
})

let unionClient: UnionClient

let toChain = derived(
  toChainId,
  $toChainId => chains.find(chain => chain.chain_id === $toChainId) ?? null
)

let fromChain = derived(
  fromChainId,
  $fromChainId => chains.find(chain => chain.chain_id === $fromChainId) ?? null
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

onMount(() => {
  fromChainId.subscribe(fromChain => {
    asset.set("")
  })

  const cosmosOfflineSigner = (
    $cosmosStore.connectedWallet === "keplr"
      ? window?.keplr?.getOfflineSigner("union-testnet-8", {
          disableBalanceCheck: false
        })
      : window.leap
        ? window.leap.getOfflineSigner("union-testnet-8", {
            disableBalanceCheck: false
          })
        : undefined
  ) as OfflineSigner

  const evmWalletClient = createWalletClient({
    chain: config.chains[0],
    account: `0x${userAddr.evm.normalized}`,
    transport: config._internal.transports["11155111"]
  })

  unionClient = new UnionClient({
    cosmosOfflineSigner,
    evmSigner: evmWalletClient,
    bech32Prefix: "union",
    chainId: "union-testnet-8",
    gas: { denom: "muno", amount: "0.0025" },
    // rpcUrl: 'https://rpc.testnet.bonlulu.uno',
    rpcUrl: "https://union-testnet-rpc.polkachu.com"
  })
})

let sendableBalances = derived(
  [fromChainId, evmBalances, cosmosBalances],
  ([$fromChainId, $evmBalances, $cosmosBalances]) => {
    if ($fromChainId === "11155111") {
      if (!$evmBalances.isSuccess) {
        console.log("trying to send from evm but no balances fetched yet")
        return null
      }
      return $evmBalances.data
    }

    const chainIndex = cosmosChains.findIndex(c => c.chain_id === $fromChainId)
    const cosmosBalance = $cosmosBalances[chainIndex]
    if (!cosmosBalance?.isSuccess || cosmosBalance.data instanceof Error) {
      console.log("trying to send from cosmos but no balances fetched yet")
      return null
    }
    return cosmosBalance.data.map(balance => ({ ...balance, balance: BigInt(balance.balance) }))
  }
)

function swapChainsClick(_event: MouseEvent) {
  const [fromChain, toChain] = [$fromChainId, $toChainId]
  toChainId.set(fromChain)
  fromChainId.set(toChain)
}

let buttonText = "Transfer" satisfies
  | "Transfer"
  | "Invalid amount"
  | "Connect Wallet"
  | "Enter an amount"
  | "Insufficient balance"
</script>

<Card.Content class={cn("flex flex-col gap-4")}>
<section>
  <CardSectionHeading>From</CardSectionHeading>
  <ChainButton bind:selectedChainId={$fromChainId} bind:dialogOpen={dialogOpenFromChain} >
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
  <Button class="size-full" variant="outline" on:click={() => (dialogOpenToken = !dialogOpenToken)}>
    <div class="flex-1 text-left">{truncate($asset, 12)}</div>

    <Chevron />
  </Button>
  {/if}
  {#if $asset !== "" && $sendableBalances !== null }
    <div class="mt-4 text-xs text-muted-foreground"><b>{truncate($asset, 12)}</b> balance on <b>{$fromChain?.display_name}</b> is <b>{$sendableBalances.find(b => b.symbol === $asset)?.balance}</b></div>
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
    disabled={!$fromChainId || !$asset || !$toChainId || !amount || !$recipient}
    on:click={async event => {
      event.preventDefault()
      const assetId = $asset
      if (!assetId) return toast.error('Please select an asset')
      if (!$fromChainId) return toast.error('Please select a from chain')
      if (!$toChainId) return toast.error('Please select a to chain')
      if (!amount) return toast.error('Please select an amount')
      if (!$recipient) return toast.error('Invalid recipient')

      toast.info(
        `Sending transaction from ${$fromChainId} to ${$fromChainId}`,
      )
      if ($fromChainId === String(sepolia.id)) {
        if ($evmAccount.status !== 'connected')
          return toast.error('Please connect your Sepolia wallet')
        if (!isAddress(assetId)) return toast.error('Invalid address')

        const evmClient = await getWalletClient(config)
        const client = new UnionClient({
          // @ts-ignore
          cosmosOfflineSigner: undefined,
          evmSigner: evmClient,
          bech32Prefix: 'union',
          chainId: 'union-testnet-8',
          gas: { denom: 'muno', amount: '0.0025' },
          rpcUrl: 'https://union-testnet-rpc.polkachu.com',
        })
        const approveHash = await client.approveEvmAssetTransfer({
          account: $evmAccount || evmClient.account,
          denomAddress: assetId,
          amount: BigInt(amount),
        })
        toast.success(`Approve transaction sent: ${approveHash}`)
        const transferHash = await client.transferEvmAsset({
          account: evmClient.account,
          receiver: $recipient,
          denomAddress: assetId,
          amount: BigInt(amount),
          sourceChannel: 'channel-1',
          simulate: true,
          contractAddress: '0xD0081080Ae8493cf7340458Eaf4412030df5FEEb',
        })
        toast.success(`Transfer transaction sent: ${transferHash}`)
      } else {
        const transferHash = await unionClient.transferAssets({
          kind: 'cosmwasm',
          instructions: [
            {
              contractAddress:
                'union1eumfw2ppz8cwl8xdh3upttzp5rdyms48kqhm30f8g9u4zwj0pprqg2vmu3',
              msg: {
                transfer: {
                  channel: 'channel-28',
                  receiver: $recipient?.slice(2),
                  memo: ``,
                },
              },
              funds: [{ denom: assetId, amount }],
            },
          ],
        })
        toast.success(`Transfer transaction sent: ${transferHash}`)
      }
    }}
  >
    {buttonText}
  </Button>
  <div class="text-muted-foreground">
    Will transfer <b>{amount} {truncate($asset, 6)}</b> from <b>{$fromChain?.display_name}</b> to <span class="font-bold font-mono">{truncate($recipient, 6)}</span> on <b>{$toChain?.display_name}</b>. 
  </div>
</Card.Footer>
<ChainDialog
  kind="from"
  {chains}
  selectedChain={$fromChainId}
  onChainSelect={(newSelectedChain) => {fromChainId.set(newSelectedChain)}}
  bind:dialogOpen={dialogOpenFromChain}
/>

<ChainDialog
  kind="to"
  {chains}
  selectedChain={$toChainId}
  onChainSelect={(newSelectedChain) => {toChainId.set(newSelectedChain)}}
  bind:dialogOpen={dialogOpenToChain}
/>

{#if $sendableBalances !== null}
  <AssetsDialog
    assets={$sendableBalances}
    onAssetSelect={(newSelectedAsset) => {asset.set(newSelectedAsset)}}
    bind:dialogOpen={dialogOpenToken}
  />
{/if}

