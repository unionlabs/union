<script lang="ts">
import { onMount } from "svelte"
import { page } from "$app/stores"
import toast from "svelte-french-toast"
import { debounce } from "$lib/utilities"
import { UnionClient } from "@union/client"
import type { PageData } from "./$types.ts"
import { queryParamStore } from "svelte-ux"
import { cn } from "$lib/utilities/shadcn.ts"
import Timer from "virtual:icons/lucide/timer"
import Chevron from "./(components)/chevron.svelte"
import Settings from "virtual:icons/lucide/settings"
import { dollarize } from "$lib/utilities/format.ts"
import type { OfflineSigner } from "@leapwallet/types"
import LockLockedIcon from "virtual:icons/lucide/lock"
import * as Card from "$lib/components/ui/card/index.ts"
import { sepoliaStore } from "$lib/wallet/evm/config.ts"
import { queryParameters } from "sveltekit-search-params"
import LockOpenIcon from "virtual:icons/lucide/lock-open"
import { Input } from "$lib/components/ui/input/index.js"
import ChainDialog from "./(components)/chain-dialog.svelte"
import ChainButton from "./(components)/chain-button.svelte"
import { cosmosStore } from "$/lib/wallet/cosmos/config.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import AssetsDialog from "./(components)/assets-dialog.svelte"
import SettingsDialog from "./(components)/settings-dialog.svelte"
import ArrowLeftRight from "virtual:icons/lucide/arrow-left-right"
import CardSectionHeading from "./(components)/card-section-heading.svelte"

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

function handleAssetSearch(event: InputEvent) {
  const target = event.target
  if (!(target instanceof HTMLInputElement)) return
  tokenSearch = target.value
  assetSearchResults = assets.filter(asset =>
    asset.id.toLowerCase().includes(tokenSearch.toLowerCase())
  )
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

const onUnlockClick = (_event: MouseEvent) =>
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

let buttonText = "Transfer" satisfies
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

<main
  class="overflow-scroll flex justify-center size-full items-start px-0 sm:px-3 max-h-full sm:py-8"
>
  <Card.Root class={cn("max-w-[475px] w-full")}>
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
        <div class="text-2xl font-bold flex-1 text-left">{selectedAsset?.symbol}</div>

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
        autocapitalize="none"
        data-transfer-from-amount
        bind:value={inputValue.from}
        pattern="^[0-9]*[.,]?[0-9]*$"
      />
      <CardSectionHeading>Recipient</CardSectionHeading>
      <div class="flex gap-2 flex-row">
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
          class={cn()}
        />
        <Button
          size="icon"
          type="button"
          variant="outline"
          name="recipient-lock"
          on:click={onUnlockClick}
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
    </Card.Content>
    <Card.Footer>
      <Button
        type="button"
        disabled={false}
        data-transfer-button=""
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
