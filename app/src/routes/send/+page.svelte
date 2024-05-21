<script lang="ts">
import { onMount } from "svelte"
import toast from "svelte-french-toast"
import { debounce } from "$lib/utilities"
import { UnionClient } from "@union/client"
import type { PageData } from "./$types.ts"
import { cn } from "$lib/utilities/shadcn.ts"
import Timer from "virtual:icons/lucide/timer"
import Settings from "virtual:icons/lucide/settings"
import { dollarize } from "$lib/utilities/format.ts"
import type { OfflineSigner } from "@leapwallet/types"
import LockLockedIcon from "virtual:icons/lucide/lock"
import * as Card from "$lib/components/ui/card/index.ts"
import { sepoliaStore } from "$lib/wallet/evm/config.ts"
import { queryParameters } from "sveltekit-search-params"
import LockOpenIcon from "virtual:icons/lucide/lock-open"
import { Input } from "$lib/components/ui/input/index.js"
import ChainDialog from "./components/chain-dialog.svelte"
import ChevronDown from "virtual:icons/lucide/chevron-down"
import { cosmosStore } from "$/lib/wallet/cosmos/config.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import AssetsDialog from "./components/assets-dialog.svelte"
import SettingsDialog from "./components/settings-dialog.svelte"
import ArrowLeftRight from "virtual:icons/lucide/arrow-left-right"
import DraftPageNotice from "$lib/components/draft-page-notice.svelte"

/**
 * TODO:
 * - [ ]
 */

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

  unionClient = new UnionClient({
    cosmosOfflineSigner,
    bech32Prefix: "union",
    chainId: "union-testnet-8",
    gas: { denom: "muno", amount: "0.0025" },
    rpcUrl: "https://rpc.testnet.bonlulu.uno"
  })
})

export let data: PageData
const { chains, assets } = data

const devBorder = 0 && "outline outline-[1px] outline-pink-200/40"

const queryParams = queryParameters(
  {
    fromChain: { encode: v => v?.toString(), decode: v => v, defaultValue: "union-testnet-8" },
    toChain: { encode: v => v?.toString(), decode: v => v, defaultValue: "11155111" },
    token: { encode: v => v?.toString(), decode: v => v, defaultValue: "union-sepolia-uno" },
    recipient: {
      encode: v => v?.toString(),
      decode: v => v,
      defaultValue: $sepoliaStore.address || ""
    }
  },
  { debounceHistory: 1_000, showDefaults: true }
)

let dialogOpenFromChain = false
let dialogOpenToChain = false
let dialogOpenToken = false
let dialogOpenSettings = false
let dialogOpenPast = false

let [chainSearch, chainSearchResults] = ["", chains]

function handleChainSearch(event: InputEvent) {
  const target = event.target
  if (!(target instanceof HTMLInputElement)) return
  chainSearch = target.value
  chainSearchResults = chains.filter(chain =>
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

let selectedFromChain = chains.find(chain => chain.id === $queryParams.fromChain)
$: selectedFromChain = chains.find(
  // chain => chain.name.toLowerCase() === $queryParams.fromChain.toLowerCase(),
  chain => chain.id === $queryParams.fromChain
)

let selectedToChain = chains.find(
  // chain => chain.name.toLowerCase() === $queryParams.toChain.toLowerCase(),
  chain => chain.id === $queryParams.toChain
)
$: selectedToChain = chains.find(
  // chain => chain.name.toLowerCase() === $queryParams.toChain.toLowerCase(),
  chain => chain.id === $queryParams.toChain
)

// console.log(JSON.stringify({ selectedFromChain, selectedToChain }, undefined, 2))

let [tokenSearch, assetSearchResults] = ["", assets]
// console.log(JSON.stringify(assets, undefined, 2))

function handleAssetSearch(event: InputEvent) {
  const target = event.target
  if (!(target instanceof HTMLInputElement)) return
  tokenSearch = target.value
  assetSearchResults = assets.filter(asset =>
    asset.id.toLowerCase().includes(tokenSearch.toLowerCase())
  )
  // console.log(JSON.stringify(assetSearchResults, undefined, 2))
}

let availableAssets = assets.filter(
  // token => token.symbol.toLowerCase() === $queryParams.token.toLowerCase(),
  asset =>
    asset.source.chain === selectedFromChain?.id && asset.destination.chain === selectedToChain?.id
)
$: availableAssets = assets.filter(
  // token => token?.symbol?.toLowerCase() === $queryParams?.token?.toLowerCase(),
  asset =>
    asset.source.chain === selectedFromChain?.id && asset.destination.chain === selectedToChain?.id
)

let selectedAsset = assets[0]
// $: console.log(JSON.stringify({ selectedAsset }, undefined, 2))

function handleAssetSelect(id: string) {
  console.log({ id }, availableAssets.find(asset => asset.id === id)?.id)
  $queryParams.token = availableAssets.find(asset => asset.id === id)?.id ?? toast("oof")
  dialogOpenToken = false
}

$: assetId = selectedAsset?.id.split("-")

const amountRegex = /[^0-9.]|\.(?=\.)|(?<=\.\d+)\./g
let inputValue = {
  from: "",
  to: "",
  recipient:
    selectedToChain?.ecosystem === "evm" && $sepoliaStore?.address
      ? $sepoliaStore?.address
      : selectedToChain?.ecosystem === "cosmos" &&
          $cosmosStore?.address &&
          $cosmosStore.address.startsWith(selectedToChain.name)
        ? $cosmosStore?.address
        : ""
}

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

let recipientInputState: "locked" | "unlocked" | "invalid" = "locked"

const onUnlockClick = (event: MouseEvent) =>
  (recipientInputState = recipientInputState === "locked" ? "unlocked" : "locked")

$: {
  // if to chain changes, update recipient address
  inputValue.recipient =
    selectedToChain?.ecosystem === "evm" && $sepoliaStore?.address
      ? $sepoliaStore?.address
      : selectedToChain?.ecosystem === "cosmos" &&
          $cosmosStore?.address &&
          $cosmosStore.address.startsWith(selectedToChain.name)
        ? $cosmosStore?.address
        : ""

  // if recipient address is locked, update it
  if (recipientInputState === "locked") {
    inputValue.recipient =
      selectedToChain?.ecosystem === "evm" && $sepoliaStore?.address
        ? $sepoliaStore?.address
        : selectedToChain?.ecosystem === "cosmos" &&
            $cosmosStore?.address &&
            $cosmosStore.address.startsWith(selectedToChain.name)
          ? $cosmosStore?.address
          : ""
  }

  if (recipientInputState === "unlocked") {
    inputValue.recipient = ""
  }
}

let buttonText = "Send it" satisfies
  | "Send"
  | "Invalid amount"
  | "Connect Wallet"
  | "Enter an amount"
  | "Insufficient balance"
  | String
</script>

<svelte:head>
  <title>Union | Send</title>
</svelte:head>

<main class="flex justify-center size-full items-start px-0 sm:px-3 max-h-full">
  <Card.Root
    class="size-full max-w-[475px] sm:mt-8 mt-6 p-2 bg-transparent border-none outline-none"
  >
    <Card.Header
      class="pt-0.5 px-2 pb-0 flex flex-row w-full justify-between items-start h-10 gap-x-3 mb-4"
    >
      <Card.Title class="text-2xl font-black mt-1">Transfer</Card.Title>
      <Button
        size="icon"
        type="button"
        variant="ghost"
        title="Ongoing transactions"
        class="size-8 bg-card text-foreground p-0 outline-1 outline-accent/80 outline rounded-xl ml-auto"
        on:click={() => (dialogOpenPast = !dialogOpenPast)}
      >
        <Timer class="size-5" />
      </Button>
      <Button
        size="icon"
        variant="ghost"
        class="size-8 bg-card text-foreground p-0 outline-1 outline-accent/80 outline rounded-xl"
        on:click={() => (dialogOpenSettings = !dialogOpenSettings)}
      >
        <Settings class="size-5" />
      </Button>
    </Card.Header>
    <Card.Content
      class={cn(
        'size-full pb-3 px-3.5 flex flex-col justify-between',
        devBorder,

        'bg-card/60 bg-opacity-60 shadow-2xl shadow-cyan-300/10 border-none outline outline-1 outline-accent/50 rounded-md',
      )}
    >
      <div
        data-transfer-from-section
        class={cn(devBorder, 'w-full pb-0 mt-4 mb-2 flex flex-row justify-between')}
      >
        <Button
          variant="ghost"
          data-transfer-from-chain=""
          on:click={() => (dialogOpenFromChain = !dialogOpenFromChain)}
          class="flex flex-row justify-between space-x-2 p-2 border-none rounded-sm size-full"
        >
          <div class="flex space-x-1.5 h-full">
            <img
              src={selectedFromChain?.icon}
              alt={`${selectedFromChain?.name} logo`}
              class="size-11 my-auto mr-auto invert dark:invert-0"
            />
            <div class="size-full mr-auto flex flex-col items-start justify-center space-y-2">
              <span class="sm:text-[1.5rem] text-xl font-extrabold mr-auto w-full text-left">
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
          class="h-8 w-16 rounded-xl dark:text-white my-auto mx-3"
          on:click={swapChainsClick}
        >
          <ArrowLeftRight class="size-5 dark:text-white" />
        </Button>

        <Button
          variant="ghost"
          data-transfer-to-chain=""
          on:click={() => (dialogOpenToChain = !dialogOpenToChain)}
          class="flex flex-row justify-between space-x-2 p-2 border-none rounded-sm size-full"
        >
          <div class="flex space-x-1.5 h-full">
            <img
              src={selectedToChain?.icon}
              alt={`${selectedToChain?.name} logo`}
              class="size-11 my-auto mr-auto"
            />
            <div class="size-full mr-auto flex flex-col items-start justify-center space-y-2">
              <span class="sm:text-[1.5rem] text-xl font-extrabold mr-auto w-full text-left">
                {selectedToChain?.name}
              </span>
              <span class="text-xs text-muted-foreground">{selectedToChain?.id}</span>
            </div>
          </div>
          <ChevronDown class="-mt-6 size-6 text-accent-foreground/60" />
        </Button>
      </div>
      <!-- asset -->
      <div class={cn('size-full mb-auto')}>
        <p class="text-left text-2xl my-2 font-extrabold ml-2">Asset</p>
        <Button
          variant="outline"
          on:click={() => (dialogOpenToken = !dialogOpenToken)}
          class={cn(
            devBorder,
            'size-full max-h-[5.5rem] flex flex-row justify-between space-x-2 px-2 pl-3 pt-1.5 border-none',
            'outline outline-1 outline-accent/90',
          )}
        >
          <div
            class={cn(
              devBorder,
              'w-full max-w-[65px] h-full mr-1 flex flex-row justify-start items-center z-50',
            )}
          >
            <img
              alt="asset"
              src={`/images/icons/${selectedFromChain?.name}.svg`}
              class={cn(
                'size-14 outline-[1.5px] outline-accent outline rounded-full bg-[#0b0b0b]',
                'p-1 z-10',
              )}
            />
            <img
              src={`/images/icons/${selectedAsset?.symbol.toLowerCase()}.svg`}
              alt="asset"
              class={cn('size-12 z-50 my-auto mt-4 -ml-8')}
            />
          </div>

          <div
            class={cn(devBorder, 'size-full max-w-[250px] flex flex-col justify-between space-y-3')}
          >
            <p class="text-2xl font-black m-auto">{selectedAsset?.symbol}</p>
            <p class="text-xs m-auto">
              {selectedAsset?.source.chain}
              {'->'}
              {selectedAsset?.destination.chain}
            </p>
          </div>
          <div class="h-full space-y-2">
            <p class="">balance</p>
            <p class={cn(devBorder, 'font-sans text-2xl font-black m-auto tabular-nums')}>--</p>
          </div>

          <ChevronDown class={cn(devBorder, 'size-6 mb-auto mt-0.5 ml-auto')} />
        </Button>
      </div>

      <!-- amount -->
      <div class={cn('my-2')}>
        <p class="text-left text-2xl font-extrabold ml-2">Amount</p>
        <Input
          minlength={1}
          maxlength={64}
          placeholder="0.00"
          autocorrect="off"
          autocomplete="off"
          spellcheck="false"
          autocapitalize="none"
          data-transfer-from-amount
          bind:value={inputValue.from}
          pattern="^[0-9]*[.,]?[0-9]*$"
          class={cn(
            'text-5xl font-bold h-20 mt-2 mb-0 px-3 tabular-nums border-none',
            'outline-1 outline-accent/80 outline',
          )}
        />
      </div>
      <!-- recipient -->
      <div class={cn('my-2')}>
        <p class="text-left text-xl font-extrabold ml-2 mb-2">Recipient</p>
        <div class="relative flex-1 mr-auto">
          <Input
            minlength={1}
            maxlength={64}
            autocorrect="off"
            autocomplete="off"
            spellcheck="false"
            autocapitalize="none"
            data-transfer-recipient-address
            placeholder="Destination address"
            bind:value={inputValue.recipient}
            disabled={recipientInputState === 'locked' && inputValue.recipient.length > 0}
            class={cn(
              inputValue.recipient.startsWith('0x') ? 'text-[0.94rem]' : 'text-[0.9rem]',
              'text-justify mt-2 mb-0 px-3 tabular-nums border-none text-balance my-auto',
              'outline-1 outline-accent/80 outline',
            )}
          />
          <Button
            size="icon"
            type="button"
            variant="ghost"
            name="recipient-lock"
            on:click={onUnlockClick}
            class="absolute bottom-[1px] right-0 rounded-l-none"
          >
            <LockLockedIcon
              class={cn(
                recipientInputState === 'locked' && inputValue.recipient.length > 0
                  ? 'size-5'
                  : 'hidden',
              )}
            />
            <LockOpenIcon
              class={cn(
                recipientInputState === 'unlocked' || !inputValue.recipient.length
                  ? 'size-5'
                  : 'hidden',
              )}
            />
          </Button>
        </div>
      </div>
    </Card.Content>
    <Card.Footer class="py-0 px-0 mt-4">
      <Button
        type="button"
        disabled={false}
        data-transfer-button=""
        class="w-full bg-secondary-foreground/90 text-xl font-bold"
        on:click={async event => {
          throw new Error('Not implemented')
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

<!-- token dialog -->
<AssetsDialog
  {handleAssetSearch}
  {handleAssetSelect}
  {assetSearchResults}
  dialogOpen={dialogOpenToken}
/>

<DraftPageNotice className="hidden sm:inline" />

<style lang="postcss">
</style>
