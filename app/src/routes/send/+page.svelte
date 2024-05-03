<script lang="ts">
import type { PageData } from "./$types.ts"
import { cn } from "$lib/utilities/shadcn.ts"
import Search from "lucide-svelte/icons/search"
import Settings from "lucide-svelte/icons/settings"
import * as Dialog from "$lib/components/ui/dialog"
import { debounce, dollarize } from "$lib/utilities"
import { queryParameters } from "sveltekit-search-params"
import * as Card from "$lib/components/ui/card/index.js"
import { Input } from "$lib/components/ui/input/index.js"
import ChevronDown from "lucide-svelte/icons/chevron-down"
import ArrowUpDown from "lucide-svelte/icons/arrow-up-down"
import { Button } from "$lib/components/ui/button/index.js"
import ArrowLeftRight from "lucide-svelte/icons/arrow-left-right"
import { Separator } from "$lib/components/ui/separator/index.js"
import DraftPageNotice from "$lib/components/draft-page-notice.svelte"

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

let dialogOpen = { fromChain: false, toChain: false, token: false }

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
  debounce(() => [($queryParams[target] = name), (dialogOpen[target] = !dialogOpen[target])], 200)()

let selectedFromChain = data.chains.find(
  chain => chain.name.toLowerCase() === $queryParams.fromChain.toLowerCase()
)
$: selectedFromChain = data.chains.find(
  chain => chain.name.toLowerCase() === $queryParams.fromChain.toLowerCase()
)

let [tokenSearch, tokenSearchResults] = ["", selectedFromChain?.assets]

function handleTokenSearch(event: InputEvent) {
  const target = event.target
  if (!(target instanceof HTMLInputElement)) return
  tokenSearch = target.value
  tokenSearchResults = selectedFromChain?.assets.filter(asset =>
    asset.symbol.toLowerCase().includes(tokenSearch.toLowerCase())
  )
}

let selectedToken = selectedFromChain?.assets.find(
  token => token.symbol.toLowerCase() === $queryParams.token.toLowerCase()
)
$: selectedToken = selectedFromChain?.assets.find(
  token => token.symbol.toLowerCase() === $queryParams.token.toLowerCase()
)

$: selectedToChain = data.chains.find(
  chain => chain.name.toLowerCase() === $queryParams.toChain.toLowerCase()
)

const amountRegex = /[^0-9.]|\.(?=\.)|(?<=\.\d+)\./g
let inputValue = { from: "", to: "" }
$: {
  inputValue.from = inputValue.from.replaceAll(amountRegex, "")
  inputValue.to = inputValue.to.replaceAll(amountRegex, "")
}

let buttonText = "Connect Wallet" satisfies
  | "Send"
  | "Invalid amount"
  | "Connect Wallet"
  | "Enter an amount"
  | "Insufficient balance"
</script>

<main class="flex justify-center items-start w-full px-0 sm:px-3 min-h-full">
  <Card.Root
    class="size-full max-w-[460px] h-[480px] border-accent border-[1px] border-solid mt-16 sm:mt-20 bg-transparent p-2"
  >
    <Card.Header class="pt-1 px-3 pb-0 flex flex-row w-full justify-between items-start h-10">
      <Card.Title class="text-2xl font-black mt-1">Transfer</Card.Title>
      <Button size="icon" variant="ghost" class="size-8 bg-card text-foreground">
        <Settings class="size-6" />
      </Button>
    </Card.Header>
    <Card.Content
      class={cn(['size-full max-h-[77%] pb-3 px-1 flex flex-col justify-between', devBorder])}
    >
      <!-- from section -->
      <div
        data-transfer-from-section
        class={cn(devBorder, 'w-full pb-0 mt-4 mb-2 h-min flex flex-row justify-between')}
      >
        <Button
          variant="ghost"
          data-transfer-from-chain
          on:click={() => (dialogOpen.fromChain = !dialogOpen.fromChain)}
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
        >
          <ArrowLeftRight class="size-5 text-white" />
        </Button>

        <Button
          variant="ghost"
          data-transfer-to-chain
          on:click={() => (dialogOpen.toChain = !dialogOpen.toChain)}
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
          class={cn(devBorder, 'size-full max-h-20 flex flex-row justify-between pl-0')}
        >
          <img src="/images/icons/osmosis.svg" alt="asset" class="size-16 mb-2" />
          <div></div>
          <ChevronDown class="size-6" />
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
          class={cn(['text-4xl h-20 mt-2 mb-0 focus-visible:ring-0'])}
        />
      </div>
      <!-- middle section -->
      <!-- <div class={cn([devBorder, 'h-min w-full mx-auto flex self-center items-center'])}>
        <Button
          size="icon"
          variant="outline"
          class={cn([devBorder, 'p-2 rounded-xl mx-auto h-9 z-10'])}
        >
          <ArrowUpDown class="size-6 rounded-full" />
        </Button>
      </div> -->

      <!-- to section -->
      <!-- 
      <div
        data-transfer-to-section
        class={cn(
          devBorder,
          'w-full p-2 pb-0 my-1 h-full max-h-[45%] flex flex-col justify-between',
        )}
      >
          <Input
          minlength={1}
          maxlength={64}
          placeholder="0.00"
          autocomplete="off"
          data-transfer-to-amount
          bind:value={inputValue.to}
          pattern="^[0-9]*[.,]?[0-9]*$"
          class={cn(['text-2xl h-14 mt-2 mb-0 focus-visible:ring-0'])}
        />
        <div class={cn([devBorder, 'flex space-x-3 w-1/2 h-22'])}>
          <Button
            variant="outline"
            class="pr-2 w-full h-full flex flex-col space-y-2 border-none pt-0"
            on:click={() => (dialogOpen.to = !dialogOpen.to)}
          >
            <div class="flex flex-row justify-between items-center w-full pl-1">
              <span class="text-lg font-bold text-zinc-200">To</span>
              <ChevronDown class="size-6" />
            </div>
            <div class="flex w-full space-x-2">
              <img
                src={selectedToChain?.icon}
                class="size-8 my-auto mr-auto"
                alt={`${selectedToChain?.name} logo`}
              />
              <div class="size-full mr-auto flex flex-col items-start space-y-0.5">
                <span
                  class="my-auto text-[22px] font-extrabold mr-auto w-full text-left justify-between"
                >
                  {selectedToChain?.name}
                </span>
                <span class="text-xs text-muted-foreground">{selectedToChain?.id}</span>
              </div>
            </div>
          </Button>
        </div> 
      </div>
    -->
    </Card.Content>
    <Card.Footer class="py-0 px-2 mt-4">
      <Button class="w-full" disabled={true}>{buttonText}</Button>
    </Card.Footer>
  </Card.Root>
</main>

<!-- from-dialog -->
<Dialog.Root
  closeOnEscape={true}
  preventScroll={true}
  closeOnOutsideClick={true}
  bind:open={dialogOpen.fromChain}
>
  <Dialog.Content
    class={cn([
      'border-solid border-accent overflow-auto flex flex-col items-start rounded-md',
      'max-w-[90%] sm:max-w-[375px] border-[1px] pt-4 pb-1 px-2',
    ])}
  >
    <Dialog.Header class="max-h-min h-16 p-2 w-full space-y-3">
      <Dialog.Title class="font-extrabold text-2xl text-center -mt-2">Select Network</Dialog.Title>
      <ul class="flex space-x-4 w-full justify-start">
        <li>all</li>
        <li>evm</li>
        <li>cosmos</li>
      </ul>
    </Dialog.Header>
    <Dialog.Description class="size-full">
      <div class="relative mr-auto flex-1 w-full">
        <Search class="absolute left-2.5 top-2.5 size-4 text-muted-foreground" />
        <Input
          type="search"
          pattern="[a-z]"
          autocorrect="off"
          spellcheck="false"
          placeholder="union…"
          autocapitalize="off"
          on:input={handleChainSearch}
          class="w-full rounded-none bg-current/95 pl-8 self-stretch lowercase border-x-0 focus-visible:ring-0"
        />
      </div>
      <ul class="my-3 mx-2 space-y-1">
        {#each chainSearchResults as { name, id: chainId, icon, live }, index}
          <li
            class={cn([
              live ? 'cursor-pointer' : 'cursor-not-allowed',
              'pb-2 dark:text-accent-foreground flex flex-col h-full justify-start align-middle space-x-3.5',
            ])}
          >
            <Button
              disabled={!live}
              on:click={() => handleChainSelect(name.toLowerCase(), 'fromChain')}
              variant={$queryParams.from === name.toLowerCase() ? 'secondary' : 'ghost'}
              class={cn([
                'w-full flex justify-start space-x-4 p-2 rounded-none pl-3 h-[55px] my-auto',
              ])}
            >
              <img src={icon} alt={`${name} logo`} class="size-10 my-auto mr-auto" />
              <div class="size-full mr-auto flex flex-col items-start">
                <span
                  class="my-auto text-[22px] font-extrabold mr-auto w-full text-left justify-between"
                >
                  {name}
                </span>
                <span class="text-xs text-muted-foreground">{chainId}</span>
              </div>
            </Button>
          </li>
        {/each}
      </ul>
    </Dialog.Description>
  </Dialog.Content>
</Dialog.Root>

<!-- token dialog -->
<!-- <Dialog.Root
  closeOnEscape={true}
  preventScroll={true}
  closeOnOutsideClick={true}
  bind:open={dialogOpen.token}
>
  <Dialog.Content
    class="max-w-[90%] sm:max-w-[350px] border-[1px] border-solid border-accent overflow-auto px-0 pt-3 pb-0 flex flex-col items-start rounded-md"
  >
    <Dialog.Header class="max-h-min h-8 p-2">
      <Dialog.Title class="font-extrabold text-2xl pl-3 -mt-2">Select a token</Dialog.Title>
    </Dialog.Header>
    <Dialog.Description class="size-full">
      <div class="relative mr-auto flex-1 w-full">
        <Search class="absolute left-2.5 top-2.5 size-4 text-muted-foreground" />
        <Input
          type="search"
          pattern="[a-z]"
          autocorrect="off"
          spellcheck="false"
          placeholder="union…"
          autocapitalize="off"
          on:input={handleTokenSearch}
          class="w-full rounded-none bg-current/95 pl-8 self-stretch lowercase border-x-0 focus-visible:ring-0"
        />
      </div>
      <ul class="my-3 mx-2 space-y-1">
        {#each tokenSearchResults as { name, id: chainId, icon, live }, index}
          <li
            class={cn([
              live ? 'cursor-pointer' : 'cursor-not-allowed',
              'pb-2 dark:text-accent-foreground flex flex-col h-full justify-start align-middle space-x-3.5',
            ])}
          >
            <Button
              disabled={!live}
              on:click={() => handleChainSelect(name.toLowerCase(), 'fromChain')}
              variant={$queryParams.from === name.toLowerCase() ? 'secondary' : 'ghost'}
              class={cn([
                'w-full flex justify-start space-x-4 p-2 rounded-none pl-3 h-[55px] my-auto',
              ])}
            >
              <img src={icon} alt={`${name} logo`} class="size-10 my-auto mr-auto" />
              <div class="size-full mr-auto flex flex-col items-start">
                <span
                  class="my-auto text-[22px] font-extrabold mr-auto w-full text-left justify-between"
                >
                  {name}
                </span>
                <span class="text-xs text-muted-foreground">{chainId}</span>
              </div>
            </Button>
          </li>
        {/each}
      </ul>
    </Dialog.Description>
  </Dialog.Content>
</Dialog.Root> -->

<svelte:head>
  <title>Union - Send</title>
</svelte:head>

<DraftPageNotice />

<style lang="postcss">
</style>
