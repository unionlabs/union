<script lang="ts">
import { onMount } from "svelte"
import { type Readable } from "svelte/store"
import { toast } from "svelte-sonner"
import { sepolia } from "viem/chains"
import { debounce } from "$lib/utilities"
import { UnionClient } from "@union/client"
import type { PageData } from "./$types.ts"
import { cn } from "$lib/utilities/shadcn.ts"
import { getWalletClient } from "@wagmi/core"
import Timer from "virtual:icons/lucide/timer"
import Chevron from "./(components)/chevron.svelte"
import Settings from "virtual:icons/lucide/settings"
import { createQuery } from "@tanstack/svelte-query"
import { createWalletClient, isAddress } from "viem"
import { evmAccount } from "$lib/wallet/evm/stores.ts"
import type { OfflineSigner } from "@leapwallet/types"
import * as Card from "$lib/components/ui/card/index.ts"
import { queryParameters } from "sveltekit-search-params"
import { Input } from "$lib/components/ui/input/index.js"
import { cosmosStore } from "$/lib/wallet/cosmos/config.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import ChainDialog from "./(components)/chain-dialog.svelte"
import ChainButton from "./(components)/chain-button.svelte"
import AssetsDialog from "./(components)/assets-dialog.svelte"
import { sepoliaStore, config } from "$lib/wallet/evm/config.ts"
import ArrowLeftRight from "virtual:icons/lucide/arrow-left-right"
import RecipientField from "./(components)/recipient-field.svelte"
import CardSectionHeading from "./(components)/card-section-heading.svelte"
import { cosmosBalancesQuery, evmBalancesQuery } from "$lib/queries/balance"
import { derived } from "svelte/store"
import { chainsQuery } from "$lib/queries/chains.ts"
    import { truncate } from "$lib/utilities/format.ts";

export let data: PageData

let evmBalances: null | ReturnType<typeof evmBalancesQuery>
$: if ($sepoliaStore.address)
  evmBalances = evmBalancesQuery({
    chainId: "11155111",
    address: $sepoliaStore.address,
    tokenSpecification: "erc20"
  })

let chains = chainsQuery()
let cosmosBalances: null | ReturnType<typeof cosmosBalancesQuery>
let cosmosChains = derived(chains, $chains => {
  if (!$chains?.isSuccess) {
    return null
  }
  return $chains.data.filter(
    (c: (typeof $chains.data)[number]) =>
      c.rpc_type === "cosmos" && c.addr_prefix !== null && c.rpcs && c.chain_id
  )
})

$: if (
  $cosmosChains &&
  $cosmosStore.rawAddress?.length !== undefined &&
  $cosmosStore.rawAddress?.length > 0
) {
  console.log($cosmosChains)
  cosmosBalances = cosmosBalancesQuery({
    // https://stackoverflow.com/questions/77206461/type-guard-function-is-not-narrowing-the-type-in-array-filter
    //@ts-ignore
    chains: $cosmosChains,
    address: $cosmosStore.rawAddress
  })


}

const queryParams = queryParameters(
  {
    "from-chain-id": {
      encode: v => v?.toString(),
      decode: v => v,
      defaultValue: "union-testnet-8"
    },
    "to-chain-id": {
      encode: v => v?.toString(),
      decode: v => v,
      defaultValue: "11155111"
    },
    recipient: { encode: v => v?.toString(), decode: v => v, defaultValue: "" },
    "asset": { encode: v => v?.toString(), decode: v => v, defaultValue: "" }
  },
  { debounceHistory: 500, showDefaults: true, sort: false }
)

let unionClient: UnionClient
onMount(() => {
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
    account: $evmAccount,
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

let dialogOpenToken = false
let dialogOpenToChain = false
let dialogOpenFromChain = false
   
const amountRegex = /[^0-9.]|\.(?=\.)|(?<=\.\d+)\./g

let amount = ""
let recipient = $queryParams.recipient || ""

$: {
  amount = amount.replaceAll(amountRegex, "")
}


let sendableBalances: null | Readable<Array<{balance: bigint, address: string, symbol: string, decimals: number}>> = null;

$:  if (queryParams && evmBalances && cosmosBalances && evmBalances !== null && cosmosBalances !== null && $cosmosChains !== null) {
 sendableBalances = derived([queryParams, evmBalances, cosmosBalances], ([$queryParams, $evmBalances, $cosmosBalances]) => {
   const fromChain = $queryParams["from-chain-id"];
   if (fromChain === "11155111") {
     if (!$evmBalances.isSuccess) {
       alert('trying to send from evm but no balances fetched yet');
       return [];
     }
    return $evmBalances.data;
   }

   const chainIndex = $cosmosChains.findIndex(c => c.chain_id === fromChain);
   const cosmosBalance = $cosmosBalances[chainIndex]
   if (!cosmosBalance.isSuccess || cosmosBalance.data instanceof Error) {
     alert('trying to send from evm but no balances fetched yet');
     return [];
   }
   return cosmosBalance.data.map((balance) => ({ ...balance, balance: BigInt(balance.balance)}))

 });
}


function swapChainsClick(_event: MouseEvent) {
  const [fromChain, toChain] = [$queryParams["from-chain-id"], $queryParams["to-chain-id"]]
  $queryParams["from-chain-id"] = toChain
  $queryParams["to-chain-id"] = fromChain
}

let buttonText = "Transfer" satisfies
  | "Transfer"
  | "Invalid amount"
  | "Connect Wallet"
  | "Enter an amount"
  | "Insufficient balance"
</script>

<svelte:head>
  <title>Union | Send</title>
</svelte:head>

{#if $chains && $chains.isSuccess && $evmBalances && $evmBalances.isSuccess && $cosmosBalances && !($cosmosBalances instanceof Error)}
<main
  class="overflow-scroll flex justify-center size-full items-start px-0 sm:px-3 max-h-full sm:py-8"
>
  <Card.Root class={cn('max-w-[490px] w-full')}>
    <Card.Header class="flex flex-row w-full items-center gap-x-2">
      <Card.Title tag="h1" class="flex-1 font-bold text-2xl">Transfer</Card.Title>
    </Card.Header>
    <Card.Content>
      <div data-transfer-from-section>
        <CardSectionHeading>From</CardSectionHeading>
        <ChainButton bind:selectedChainId={$queryParams["from-chain-id"]} bind:dialogOpen={dialogOpenFromChain} />

        <div class="flex flex-col items-center pt-4">
          <Button size="icon" variant="outline" on:click={swapChainsClick}>
            <ArrowLeftRight class="size-5 dark:text-white rotate-90" />
          </Button>
        </div>

        <CardSectionHeading>To</CardSectionHeading>
        <ChainButton bind:selectedChainId={$queryParams["to-chain-id"]} bind:dialogOpen={dialogOpenToChain} />
      </div>
      <!-- asset -->
      <CardSectionHeading>Asset</CardSectionHeading>
      {#if sendableBalances !== null}
        <div>{JSON.stringify($sendableBalances)}</div>
      {/if}
      <Button class="size-full" variant="outline" on:click={() => (dialogOpenToken = !dialogOpenToken)}>
        <div class="flex-1 text-left">{truncate($queryParams['asset'], 12)}</div>

        <Chevron />
      </Button>

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
      <CardSectionHeading>Recipient</CardSectionHeading>

      <RecipientField recipient={$queryParams.recipient} />
    </Card.Content>
    <Card.Footer>
      <Button
        type="button"
        disabled={false}
        on:click={async event => {
          event.preventDefault()
          const assetId = $queryParams['asset']
          if (!assetId) return toast.error('Please select an asset')
          toast.info(
            `Sending transaction from ${$queryParams['from-chain-id']} to ${$queryParams['to-chain-id']}`,
          )
          if ($queryParams['from-chain-id'] === String(sepolia.id)) {
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
              receiver: recipient,
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
                    'union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7',
                  msg: {
                    transfer: {
                      channel: 'channel-0',
                      receiver: recipient.slice(2),
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
    </Card.Footer>
  </Card.Root>
</main>


<!-- from-dialog -->
<ChainDialog
  kind="from"
  chains={$chains.data}
  selectedChain={$queryParams["from-chain-id"]}
  onChainSelect={(newSelectedChain) => {$queryParams["from-chain-id"] = newSelectedChain}}
  bind:dialogOpen={dialogOpenFromChain}
/>

<!-- to-dialog -->
<ChainDialog
  kind="to"
  chains={$chains.data}
  selectedChain={$queryParams["to-chain-id"]}
  onChainSelect={(newSelectedChain) => {$queryParams["to-chain-id"] = newSelectedChain}}
  bind:dialogOpen={dialogOpenToChain}
/>

  <!-- token dialog -->
  {#if $sendableBalances}
    <AssetsDialog
      assets={$sendableBalances}
      onAssetSelect={(newSelectedAsset) => {$queryParams["asset"] = newSelectedAsset}}
      bind:dialogOpen={dialogOpenToken}
    />
  {/if}
{/if}
