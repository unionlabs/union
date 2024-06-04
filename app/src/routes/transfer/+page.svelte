<script lang="ts">
import { onMount } from "svelte"
import { page } from "$app/stores"
import { toast } from "svelte-sonner"
import { debounce } from "$lib/utilities"
import { UnionClient } from "@union/client"
import type { PageData } from "./$types.ts"
import { cn } from "$lib/utilities/shadcn.ts"
import Timer from "virtual:icons/lucide/timer"
import Chevron from "./(components)/chevron.svelte"
import Settings from "virtual:icons/lucide/settings"
import { dollarize } from "$lib/utilities/format.ts"
import type { OfflineSigner } from "@leapwallet/types"
import * as Card from "$lib/components/ui/card/index.ts"
import { queryParameters } from "sveltekit-search-params"
import { Input } from "$lib/components/ui/input/index.js"
import { evmBalancesQuery } from "$lib/queries/balance.ts"
import { cosmosStore } from "$/lib/wallet/cosmos/config.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import ChainDialog from "./(components)/chain-dialog.svelte"
import ChainButton from "./(components)/chain-button.svelte"
import AssetsDialog from "./(components)/assets-dialog.svelte"
import SettingsDialog from "./(components)/settings-dialog.svelte"
import ArrowLeftRight from "virtual:icons/lucide/arrow-left-right"
import RecipientField from "./(components)/recipient-field.svelte"
import CardSectionHeading from "./(components)/card-section-heading.svelte"
import { sepoliaStore, config } from "$lib/wallet/evm/config.ts"
import { evmAccount, evmProvider, evmClient } from "$lib/wallet/evm/stores.ts"
import { getWalletClient } from "@wagmi/core"
import { createQuery } from "@tanstack/svelte-query"
import { isValidCosmosAddress } from "$lib/wallet/utilities/validate.ts"
import { isAddress } from "viem"

export let data: PageData
const { chains } = data

const devBorder = 0 && "outline outline-[1px] outline-pink-200/40"

const queryParams = queryParameters(
  {
    "from-chain-id": {
      encode: v => v?.toString(),
      decode: v => v,
      defaultValue: "11155111"
    },
    "to-chain-id": {
      encode: v => v?.toString(),
      decode: v => v,
      defaultValue: "union-testnet-8"
    },
    recipient: { encode: v => v?.toString(), decode: v => v, defaultValue: "" },
    "asset-id": { encode: v => v?.toString(), decode: v => v, defaultValue: "" }
  },
  { debounceHistory: 500, showDefaults: true, sort: false }
)

let unionClient: UnionClient
onMount(async () => {
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

  const evmWalletClient = await getWalletClient(config)

  unionClient = new UnionClient({
    cosmosOfflineSigner,
    evmSigner: evmWalletClient,
    bech32Prefix: "union",
    chainId: "union-testnet-8",
    gas: { denom: "muno", amount: "0.0025" },
    rpcUrl: "https://rpc.testnet.bonlulu.uno"
  })
})

let dialogOpenPast = false
let dialogOpenToken = false
let dialogOpenToChain = false
let dialogOpenFromChain = false
let dialogOpenSettings = false

let [chainSearch, chainSearchResults] = ["", chains]

function handleChainSearch(event: InputEvent) {
  const target = event.target
  if (!(target instanceof HTMLInputElement)) return
  chainSearch = target.value
  chainSearchResults = chains.filter(chain =>
    chain.name.toLowerCase().includes(chainSearch.toLowerCase())
  )
}

const handleChainSelect = (name: string, target: "from-chain-id" | "to-chain-id") =>
  debounce(
    () => [
      ($queryParams[target] = name),
      ([dialogOpenFromChain, dialogOpenToChain, dialogOpenToken, dialogOpenSettings] = [
        false,
        false,
        false,
        false
      ])
    ],
    200
  )()

let selectedFromChain = chains.find(chain => chain.id === $queryParams["from-chain-id"])
$: selectedFromChain = chains.find(chain => chain.id === $queryParams["from-chain-id"])

let selectedToChain = chains.find(chain => chain.id === $queryParams["to-chain-id"])
$: selectedToChain = chains.find(chain => chain.id === $queryParams["to-chain-id"])

$: unionBalances = createQuery({
  queryKey: [$cosmosStore.address, "balance", "union-testnet-8"],
  queryFn: async () => {
    const balances = await fetch(
      `https://api.testnet.bonlulu.uno/cosmos/bank/v1beta1/balances/${$cosmosStore.address}`
    )
    // const data = await
  },
  enabled: isValidCosmosAddress($cosmosStore.address)
})

$: evmBalances = evmBalancesQuery({
  address: `${$sepoliaStore.address}` as any,
  chainId: "11155111",
  tokenSpecification: "erc20"
})

$: sepoliaAssets = $evmBalances?.data ?? []

$: [tokenSearch, assetSearchResults] = ["", sepoliaAssets]

$: console.info(tokenSearch)

function handleAssetSearch(event: InputEvent) {
  const target = event.target
  if (!(target instanceof HTMLInputElement)) return
  tokenSearch = target.value
  assetSearchResults = sepoliaAssets.filter(asset =>
    asset.symbol.toLowerCase().includes(tokenSearch.toLowerCase())
  )
}

// let availableAssets = assets.filter(
//   asset =>
//     asset.source.chain === selectedFromChain?.id &&
//     asset.destination.chain === selectedToChain?.id,
// )
// $: availableAssets = assets.filter(
//   asset =>
//     asset.source.chain === selectedFromChain?.id &&
//     asset.destination.chain === selectedToChain?.id,
// )

// let selectedAsset = assets[0]

function handleAssetSelect(id: string) {
  $queryParams["asset-id"] = sepoliaAssets.find(asset => asset.symbol === id)?.address ?? ""
  dialogOpenToken = false
}

const amountRegex = /[^0-9.]|\.(?=\.)|(?<=\.\d+)\./g

let amount = ""
let recipient = $queryParams.recipient || ""

$: {
  amount = amount.replaceAll(amountRegex, "")
}

function swapChainsClick(_event: MouseEvent) {
  const [fromChain, toChain] = [$queryParams["from-chain-id"], $queryParams["to-chain-id"]]
  $queryParams["from-chain-id"] = toChain
  $queryParams["to-chain-id"] = fromChain

  selectedFromChain = data.chains.find(
    chain => chain.name.toLowerCase() === $queryParams["from-chain-id"].toLowerCase()
  )
  selectedToChain = data.chains.find(
    chain => chain.name.toLowerCase() === $queryParams["to-chain-id"].toLowerCase()
  )
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

<main
  class="overflow-scroll flex justify-center size-full items-start px-0 sm:px-3 max-h-full sm:py-8"
>
  <Card.Root class={cn('max-w-[490px] w-full')}>
    <Card.Header class="flex flex-row w-full items-center gap-x-2">
      <Card.Title tag="h1" class="flex-1 font-bold text-2xl">Transfer</Card.Title>
      <Button
        size="icon"
        type="button"
        variant="ghost"
        title="Ongoing transactions"
        on:click={() => (dialogOpenPast = !dialogOpenPast)}
        class="size-8  text-foreground p-0 outline-1 outline-accent/80 outline"
      >
        <Timer class="size-5" />
      </Button>
      <Button
        size="icon"
        variant="ghost"
        on:click={() => (dialogOpenSettings = !dialogOpenSettings)}
        class="size-8 bg-card text-foreground p-0 outline-1 outline-accent/80 outline"
      >
        <Settings class="size-5" />
      </Button>
    </Card.Header>
    <Card.Content>
      <div data-transfer-from-section>
        <CardSectionHeading>From</CardSectionHeading>
        <ChainButton bind:selectedChain={selectedFromChain} bind:dialogOpen={dialogOpenFromChain} />

        <div class="flex flex-col items-center pt-4">
          <Button size="icon" variant="outline" on:click={swapChainsClick}>
            <ArrowLeftRight class="size-5 dark:text-white rotate-90" />
          </Button>
        </div>

        <CardSectionHeading>To</CardSectionHeading>
        <ChainButton bind:selectedChain={selectedToChain} bind:dialogOpen={dialogOpenToChain} />
      </div>
      <!-- asset -->
      <CardSectionHeading>Asset</CardSectionHeading>
      <Button variant="outline" on:click={() => (dialogOpenToken = !dialogOpenToken)}>
        <div class="text-2xl font-bold flex-1 text-left">
          {sepoliaAssets.find(i => i.address === $queryParams['asset-id'])?.symbol ||
            'Select an asset'}
        </div>
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
          const assetId = $queryParams['asset-id']
          if (!assetId) return toast.error('Please select an asset')
          if ($queryParams['to-chain-id'] === '111155111') {
            if (!isAddress(assetId)) return toast.error('Invalid address')

            const evmClient = await getWalletClient(config)
            const client = new UnionClient({
              // @ts-ignore
              cosmosOfflineSigner: undefined,
              evmSigner: evmClient,
              bech32Prefix: 'union',
              chainId: 'union-testnet-8',
              gas: { denom: 'muno', amount: '0.0025' },
              rpcUrl: 'https://rpc.testnet.bonlulu.uno',
            })
            const approveHash = await client.approveEvmAssetTransfer({
              assetContractAddress: assetId,
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

<!-- settings dialog -->
<SettingsDialog dialogOpen={dialogOpenSettings} title="Preferences" />

<!-- past dialog -->
<SettingsDialog dialogOpen={dialogOpenPast} title="Past" />

<!-- from-dialog -->
<ChainDialog
  kind="from"
  {handleChainSearch}
  {handleChainSelect}
  {chainSearchResults}
  queryParams={$queryParams}
  bind:dialogOpen={dialogOpenFromChain}
/>

<!-- to-dialog -->
<ChainDialog
  kind="to"
  {handleChainSearch}
  {handleChainSelect}
  {chainSearchResults}
  queryParams={$queryParams}
  bind:dialogOpen={dialogOpenToChain}
/>

<!-- token dialog -->
<AssetsDialog
  {handleAssetSearch}
  {handleAssetSelect}
  {assetSearchResults}
  bind:dialogOpen={dialogOpenToken}
/>
