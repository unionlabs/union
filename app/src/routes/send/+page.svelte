<script lang="ts">
import { onMount } from "svelte"
import { debounce, dollarize } from "$lib/utilities"
import { UnionClient } from "@union/client"
import type { PageData } from "./$types.ts"
import { cn } from "$lib/utilities/shadcn.ts"
import Search from "virtual:icons/lucide/search"
import Settings from "virtual:icons/lucide/settings"
import type { OfflineSigner } from "@leapwallet/types"
import * as Card from "$lib/components/ui/card/index.js"
import { queryParameters } from "sveltekit-search-params"
import { Input } from "$lib/components/ui/input/index.js"
import ChevronDown from "virtual:icons/lucide/chevron-down"
import { Button } from "$lib/components/ui/button/index.js"
import ArrowLeftRight from "virtual:icons/lucide/arrow-left-right"
import DraftPageNotice from "$lib/components/draft-page-notice.svelte"
import { ChainDialog, SettingsDialog, AssetsDialog } from "$lib/components/send/dialogs/index.ts"

let unionClient: UnionClient
onMount(() => {
  const cosmosOfflineSigner = (
    window.keplr
      ? window.keplr.getOfflineSigner("union-testnet-8", {})
      : window.leap
        ? window.leap.getOfflineSigner("union-testnet-8", {})
        : undefined
  ) as OfflineSigner

  unionClient = new UnionClient({
    bech32Prefix: "union",
    chainId: "union-testnet-8",
    gas: { denom: "muno", amount: "0.0025" },
    rpcUrl: "https://rpc.testnet.bonlulu.uno",
    offlineSigner: cosmosOfflineSigner
  })
})

export let data: PageData

const devBorder = 0 && "outline outline-[1px] outline-pink-200/40"

const queryParams = queryParameters(
  {
    fromChain: { encode: v => v?.toString(), decode: v => v, defaultValue: "union" },
    toChain: { encode: v => v?.toString(), decode: v => v, defaultValue: "sepolia" },
    token: { encode: v => v?.toString(), decode: v => v, defaultValue: "uno" }
  },
  { debounceHistory: 1_000, showDefaults: true }
)

let dialogOpenFromChain = false
let dialogOpenToChain = false
let dialogOpenToken = false
let dialogOpenSettings = false

let [chainSearch, chainSearchResults] = ["", data.chains]

function handleChainSearch(event: InputEvent) {
  const target = event.target
  if (!(target instanceof HTMLInputElement)) return
  chainSearch = target.value
  chainSearchResults = data.chains.filter(chain =>
    chain.name.toLowerCase().includes(chainSearch.toLowerCase())
  )
}

const handleChainSelect = (name: string, target: "fromChain" | "toChain") =>
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

let selectedFromChain = data.chains.find(
  chain => chain.name.toLowerCase() === $queryParams.fromChain.toLowerCase()
)
$: selectedFromChain = data.chains.find(
  chain => chain.name.toLowerCase() === $queryParams.fromChain.toLowerCase()
)

let selectedToChain = data.chains.find(
  chain => chain.name.toLowerCase() === $queryParams.toChain.toLowerCase()
)
$: selectedToChain = data.chains.find(
  chain => chain.name.toLowerCase() === $queryParams.toChain.toLowerCase()
)

let [tokenSearch, tokenSearchResults] = ["", selectedFromChain?.assets]

function handleAssetSearch(event: InputEvent) {
  const target = event.target
  if (!(target instanceof HTMLInputElement)) return
  tokenSearch = target.value
  tokenSearchResults = selectedFromChain?.assets.filter(asset =>
    asset.symbol.toLowerCase().includes(tokenSearch.toLowerCase())
  )
}

let selectedAsset = selectedFromChain?.assets.find(
  token => token.symbol.toLowerCase() === $queryParams.token.toLowerCase()
)
$: selectedAsset = selectedFromChain?.assets.find(
  token => token?.symbol?.toLowerCase() === $queryParams?.token?.toLowerCase()
)

function handleAssetSelect(symbol: string) {
  $queryParams.token = symbol
  dialogOpenToken = false
}

const amountRegex = /[^0-9.]|\.(?=\.)|(?<=\.\d+)\./g
let inputValue = { from: "", to: "" }
$: {
  inputValue.from = inputValue.from.replaceAll(amountRegex, "")
  inputValue.to = inputValue.to.replaceAll(amountRegex, "")
}

function swapChainsClick(_event: MouseEvent) {
  const [fromChain, toChain] = [$queryParams.fromChain, $queryParams.toChain]
  $queryParams.fromChain = toChain
  $queryParams.toChain = fromChain

  selectedFromChain = data.chains.find(
    chain => chain.name.toLowerCase() === $queryParams.fromChain.toLowerCase()
  )
  selectedToChain = data.chains.find(
    chain => chain.name.toLowerCase() === $queryParams.toChain.toLowerCase()
  )
}

let buttonText = "Send it 🔥" satisfies
  | "Send"
  | "Invalid amount"
  | "Connect Wallet"
  | "Enter an amount"
  | "Insufficient balance"
  | String
</script>

<main class="flex justify-center items-start w-full px-0 sm:px-3 min-h-full">
  <Card.Root
    class="size-full max-w-[460px] h-[480px] border-accent border-[1px] border-solid mt-16 bg-transparent p-2"
  >
    <Card.Header class="pt-1 px-3 pb-0 flex flex-row w-full justify-between items-start h-10">
      <Card.Title class="text-2xl font-black mt-1">Transfer</Card.Title>
      <Button
        size="icon"
        variant="ghost"
        class="size-8 bg-card text-foreground"
        on:click={() => (dialogOpenSettings = !dialogOpenSettings)}
      >
        <Settings class="size-6" />
      </Button>
    </Card.Header>
    <Card.Content
      class={cn(['size-full max-h-[77%] pb-3 px-1 flex flex-col justify-between', devBorder])}
    >
      <div
        data-transfer-from-section
        class={cn(devBorder, 'w-full pb-0 mt-4 mb-2 h-min flex flex-row justify-between')}
      >
        <Button
          variant="ghost"
          data-transfer-from-chain=""
          on:click={() => (dialogOpenFromChain = !dialogOpenFromChain)}
          class="flex flex-row justify-between space-x-2 px-2 py-2 border-none rounded-sm size-full"
        >
          <div class="flex space-x-1.5 h-full">
            <img
              src={selectedFromChain?.icon}
              class="size-11 my-auto mr-auto"
              alt={`${selectedFromChain?.name} logo`}
            />
            <div class="size-full mr-auto flex flex-col items-start justify-center space-y-2">
              <span class="text-[1.5rem] font-extrabold mr-auto w-full text-left">
                {selectedFromChain?.name}
              </span>
              <span class="text-xs text-muted-foreground">{selectedFromChain?.id}</span>
            </div>
          </div>
          <ChevronDown class="-mt-6 size-6 text-accent-foreground/60" />
        </Button>

        <Button
          size="icon"
          variant="outline"
          class="size-full max-w-8 max-h-8 rounded-xl text-white my-auto mx-3"
          on:click={swapChainsClick}
        >
          <ArrowLeftRight class="size-5 text-white" />
        </Button>

        <Button
          variant="ghost"
          data-transfer-to-chain=""
          on:click={() => (dialogOpenToChain = !dialogOpenToChain)}
          class="flex flex-row justify-between space-x-2 px-2 py-2 border-none rounded-sm size-full"
        >
          <div class="flex space-x-1.5 h-full">
            <img
              src={selectedToChain?.icon}
              class="size-11 my-auto mr-auto"
              alt={`${selectedToChain?.name} logo`}
            />
            <div class="size-full mr-auto flex flex-col items-start justify-center space-y-2">
              <span class="text-[1.5rem] font-extrabold mr-auto w-full text-left">
                {selectedToChain?.name}
              </span>
              <span class="text-xs text-muted-foreground">{selectedToChain?.id}</span>
            </div>
          </div>
          <ChevronDown class="-mt-6 size-6 text-accent-foreground/60" />
        </Button>
      </div>
      <!-- asset -->
      <div class={cn('size-full h-20 max-h-20 mt-1 mb-auto')}>
        <p class="text-center text-2xl mb-2 font-extrabold">Asset</p>
        <Button
          variant="outline"
          on:click={() => (dialogOpenToken = !dialogOpenToken)}
          class={cn(
            devBorder,
            'size-full max-h-20 flex flex-row justify-between space-x-2 px-2 pl-3 pt-1.5',
          )}
        >
          <!-- <img src="/images/icons/osmosis.svg" alt="asset" class={cn('size-12 z-50 my-auto')} /> -->
          <div
            class={cn([
              devBorder,
              'w-full max-w-[65px] h-full mr-1 flex flex-row justify-start items-center',
            ])}
          >
            <img
              alt="asset"
              src="/images/icons/union.svg"
              class={cn(
                'size-12 outline-[1.5px] outline-accent outline rounded-full bg-[#0b0b0b]',
                'p-1 z-10',
              )}
            />
            <img
              src="/images/icons/osmosis.svg"
              alt="asset"
              class={cn('size-10 z-50 my-auto mt-4 -ml-6')}
            />
          </div>

          <div class={cn([devBorder, 'size-full max-w-[250px] flex flex-col'])}>
            <p class="text-2xl font-black m-auto">OSMO</p>
            <p class="text-xl m-auto">Osmosis</p>
          </div>
          <div class="h-full">
            <p class={cn([devBorder, 'font-sans text-2xl font-black m-auto tabular-nums'])}>
              420.69
            </p>
            <p class="">balance</p>
          </div>

          <ChevronDown class={cn([devBorder, 'size-6 mb-auto mt-0.5 ml-auto'])} />
        </Button>
      </div>
      <div class={cn(['mt-1'])}>
        <p class="text-center text-2xl mb-2 font-extrabold">Amount</p>
        <Input
          minlength={1}
          maxlength={64}
          placeholder="0.00"
          autocomplete="off"
          data-transfer-from-amount
          bind:value={inputValue.from}
          pattern="^[0-9]*[.,]?[0-9]*$"
          class={cn(['text-5xl font-bold h-20 mt-2 mb-0 px-3 focus-visible:ring-0 tabular-nums'])}
        />
      </div>
    </Card.Content>
    <Card.Footer class="py-0 px-2 mt-4">
      <Button
        type="button"
        class="w-full"
        disabled={false}
        data-transfer-button
        on:click={async event => {
          console.log(inputValue.from)
          const amount = parseFloat(inputValue.from)
          const account = await unionClient.getAccount()
          const contractAddress = 'union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7'

          const transfers = await unionClient.transferAssets({
            kind: 'cosmwasm',
            instructions: [
              {
                contractAddress,
                msg: {
                  transfer: {
                    channel: 'channel-5',
                    receiver: 'osmo14qemq0vw6y3gc3u3e0aty2e764u4gs5l32ydm0',
                    memo: 'sending wrapped OSMO from Union to Osmosis through the App',
                  },
                },
                funds: [
                  {
                    amount: amount.toString(),
                    denom: `factory/${contractAddress}/0xc5775fca1b3285dc8b749d58b227527211c108b8d3`,
                  },
                ],
              },
            ],
          })
          console.log(transfers.transactionHash)
        }}>{buttonText}</Button
      >
    </Card.Footer>
  </Card.Root>
</main>

<!-- from-dialog -->
<ChainDialog
  kind="from"
  {handleChainSearch}
  {handleChainSelect}
  {chainSearchResults}
  queryParams={$queryParams}
  dialogOpen={dialogOpenFromChain}
/>

<!-- to-dialog -->
<ChainDialog
  kind="to"
  {handleChainSearch}
  {handleChainSelect}
  {chainSearchResults}
  queryParams={$queryParams}
  dialogOpen={dialogOpenToChain}
/>

<!-- settings dialog -->
<SettingsDialog dialogOpen={dialogOpenSettings} />

<!-- token dialog -->
<AssetsDialog
  dialogOpen={//
  // true
  dialogOpenToken}
  {handleAssetSearch}
  {handleAssetSelect}
/>

<svelte:head>
  <title>Union - Send</title>
</svelte:head>

<DraftPageNotice />

<style lang="postcss">
</style>
