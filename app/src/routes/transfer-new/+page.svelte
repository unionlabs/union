<script lang="ts">
import {
  createQuery,
  queryOptions,
  QueryObserver,
  useQueryClient,
  keepPreviousData
} from "@tanstack/svelte-query"
import {
  http,
  type ChainId,
  createPfmMemo,
  type EvmChainId,
  createUnionClient,
  type CosmosChainId,
  evmChainFromChainId,
  bech32ToBech32Address,
  type AptosBrowserWallet,
  type TransferAssetsParameters
} from "@unionlabs/client"
import * as v from "valibot"
import { page } from "$app/stores"
import { toast } from "svelte-sonner"
import { goto } from "$app/navigation"
import { onDestroy, onMount } from "svelte"
import { cn } from "$lib/utilities/shadcn.ts"
import { userAddrEvm } from "$lib/wallet/evm"
import { config } from "$lib/wallet/evm/config"
import { toIsoString } from "$lib/utilities/date"
import { truncate } from "$lib/utilities/format.ts"
import { userAddrCosmos } from "$lib/wallet/cosmos"
import Chevron from "./(components)/chevron.svelte"
import { userBalancesQuery } from "$lib/queries/balance"
import * as Card from "$lib/components/ui/card/index.ts"
import type { Chain, UserAddresses } from "$lib/types.ts"
import { Input } from "$lib/components/ui/input/index.js"
import { userAddrOnChain } from "$lib/utilities/address.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import ChainDialog from "./(components)/chain-dialog.svelte"
import ChainButton from "./(components)/chain-button.svelte"
import AssetsDialog from "./(components)/assets-dialog.svelte"
import { getSupportedAsset } from "$lib/utilities/helpers.ts"
import { debounce, raise, sleep } from "$lib/utilities/index.ts"
import ArrowLeftRight from "virtual:icons/lucide/arrow-left-right"
import { transferSchema, type TransferSchema } from "./validation.ts"
import { getCosmosChainInfo } from "$lib/wallet/cosmos/chain-info.ts"
import { submittedTransfers } from "$lib/stores/submitted-transfers.ts"
import { parseUnits, formatUnits, type HttpTransport, getAddress } from "viem"
import { aptosStore, userAddressAptos, getAptosWallet } from "$lib/wallet/aptos"
import { cosmosStore, getCosmosOfflineSigner } from "$/lib/wallet/cosmos/config.ts"
import { type Writable, writable, derived, get, type Readable } from "svelte/store"
import { custom, switchChain, getConnectorClient, waitForTransactionReceipt } from "@wagmi/core"
import { data } from "autoprefixer"

const querClient = useQueryClient()

function queryData<T extends Array<unknown>>(
  key: Array<string>,
  filter?: (value: T[number]) => boolean
): T {
  const data = querClient.getQueryData<T>(key) ?? []
  return (filter ? data.filter(filter) : data) as T
}

let chains = queryData<Array<Chain>>(["chains"], chain => chain.enabled_staging)

/**
 * it's not useful to strongly type this becuse it refers to the raw state of the URL
 * because at this point we haven't yet parsed the URL or validated it, so it's just a string
 */
type SearchParams = { [key: string]: string }

let transferQueryOptions = queryOptions<SearchParams>({
  enabled: query => false,
  placeholderData: keepPreviousData,
  queryKey: ["transfer-state", $page.url.toString()],
  initialData: Object.fromEntries($page.url.searchParams),
  queryFn: ({ queryKey, signal, meta }) => Object.fromEntries($page.url.searchParams)
})

$: transferState = createQuery(transferQueryOptions)

const observer = new QueryObserver<SearchParams>(querClient, {
  enabled: query => false,
  queryKey: ["transfer-state"]
})

let userAddress = derived(
  [userAddrCosmos, userAddrEvm, userAddressAptos],
  ([$cosmos, $evm, $aptos]) => ({ evm: $evm, aptos: $aptos, cosmos: $cosmos })
)

$: sourceChain = writable(chains.find(chain => chain.chain_id === $transferState.data?.source))
$: destinationChain = writable(
  chains.find(chain => chain.chain_id === $transferState.data?.destination)
)

$: rawBalances = userBalancesQuery({
  chains,
  connected: true,
  userAddr: $userAddress
})

$: balances = derived([rawBalances, sourceChain], ([$rawBalances, $sourceChain]) => {
  if (!($sourceChain && $rawBalances)) return []
  return $rawBalances.data[$sourceChain.chain_id] ?? []
})

// @ts-ignore
$: assetInfo = $balances.find(x => x?.address === $transferState.data?.asset)

/**
 * observer observs the transfer state and updates the url accordingly
 */
const unsubscribe = observer.subscribe(result => {
  if (!result.data) return
  const entry = Object.entries(result.data).at(-1)
  const [key, value] = entry ?? []
  if (!key) return
  const searchParams = new URLSearchParams({ ...$page.url.searchParams, ...result.data })
  if (value?.length === 0) searchParams.delete(key)
  goto(`${$page.url.pathname}?${searchParams.toString()}`, {
    keepFocus: true,
    replaceState: true
  })

  $transferState.refetch()
})

const updateParams = (kv: SearchParams) => {
  querClient.setQueryData<SearchParams>(["transfer-state"], _ => ({ ..._, ...kv }))
}

function swapChainsClick(_event: MouseEvent) {
  if (!($transferState.data?.source && $transferState.data?.destination)) return
  updateParams({
    source: $transferState.data?.destination,
    destination: $transferState.data?.source,
    asset: ""
  })
}

let [dialogOpenFromChain, dialogOpenToChain, dialogOpenAsset] = [false, false, false]

let errors = writable([])

onDestroy(() => unsubscribe())
</script>

<ChainDialog
  {chains}
  kind="from"
  dialogOpen={dialogOpenFromChain}
  onChainSelect={value => updateParams({ source: value })}
  selectedChain={$page.url.searchParams.get('source') || $transferState.data?.source}
/>

<ChainDialog
  {chains}
  kind="to"
  selectedChain={$page.url.searchParams.get('destination') || $transferState.data?.destination}
  dialogOpen={dialogOpenToChain}
  onChainSelect={value => {
    // set receiver to self initially
    const selectedDestinationChain = chains.find(c => c.chain_id === value)
    if (!selectedDestinationChain) return
    destinationChain.set(selectedDestinationChain)
    let destinationAddress = $userAddress[$destinationChain?.rpc_type]?.canonical
    if (!destinationAddress?.length) return
    if ($destinationChain?.rpc_type === 'cosmos') {
      destinationAddress = bech32ToBech32Address({
        address: destinationAddress,
        toPrefix: $destinationChain.addr_prefix,
      })
    }
    if (!destinationAddress?.length) return updateParams({ destination: value })
    const receiverField = document.querySelector('input[name="receiver"]')
    if (!receiverField) return
    receiverField.value = destinationAddress
    return updateParams({ destination: value, receiver: destinationAddress })
  }}
/>

<AssetsDialog
  assets={$balances}
  chain={$sourceChain}
  bind:dialogOpen={dialogOpenAsset}
  onAssetSelect={({ symbol, address }) => updateParams({ asset: address })}
/>

<form
  id="transfer"
  name="transfer"
  action="transfer"
  data-form="transfer"
  on:submit={event => {
    event.preventDefault()
    event.stopPropagation()
    const final = v.safeParse(transferSchema, {
      asset: $transferState.data?.asset,
      amount: $transferState.data?.amount,
      source: $transferState.data?.source,
      receiver: $transferState.data?.receiver,
      destination: $transferState.data?.destination,
    })
    console.info(final)
    if (!final.success) {
      const issues = final.issues.map(issue => ({ path: issue.path?.at(0), ...issue }))
      // @ts-ignore
      errors.set(issues)
      toast.error(JSON.stringify($errors, undefined, 2))
    }
  }}
>
  <Card.Root class={cn('w-[500px]')}>
    <Card.Header>header</Card.Header>
    <Card.Content class={cn('flex flex-col gap-4')}>
      <section>
        <h2 class="card-section-heading">From</h2>
        <ChainButton bind:dialogOpen={dialogOpenFromChain}>
          {$sourceChain?.display_name ?? 'Select chain'}
        </ChainButton>
        <div class="flex flex-col items-center pt-4 -mb-6">
          <Button on:click={swapChainsClick} size="icon" variant="outline">
            <ArrowLeftRight class="size-5 dark:text-white rotate-90" />
          </Button>
        </div>
        <h2 class="card-section-heading">to</h2>

        <ChainButton bind:dialogOpen={dialogOpenToChain}>
          {$destinationChain?.display_name ?? 'Select chain'}
        </ChainButton>
      </section>
      <Button
        class="w-full"
        variant="outline"
        on:click={() => (dialogOpenAsset = !dialogOpenAsset)}
      >
        <div class="flex-1 text-left font-bold text-md">
          {assetInfo?.symbol ?? 'Select Asset'}
        </div>
        <Chevron />
      </Button>
      receiver
      <Input
        type="text"
        id="receiver"
        name="receiver"
        required={true}
        disabled={false}
        autocorrect="off"
        spellcheck="false"
        autocomplete="off"
        data-field="receiver"
        class="disabled:bg-black/30"
        placeholder="destination address"
        on:input={event =>
          debounce(() => updateParams({ receiver: `${event.target?.value}` }), 1_000)()}
      />
      amount
      <Input
        id="amount"
        type="number"
        name="amount"
        minlength={1}
        maxlength={64}
        required={true}
        disabled={false}
        autocorrect="off"
        placeholder="0.00"
        spellcheck="false"
        autocomplete="off"
        inputmode="decimal"
        data-field="amount"
        autocapitalize="none"
        pattern="^[0-9]*[.,]?[0-9]*$"
        class={cn('focus:ring-0 focus-visible:ring-0 disabled:bg-black/30')}
        on:input={event =>
          debounce(() => updateParams({ amount: `${event.target?.value}` }), 1_000)()}
      />
    </Card.Content>

    <Card.Footer class="flex flex-col gap-4 items-start">
      <Button
        type="submit"
        form="transfer"
        disabled={false}
        id="transfer-button"
        data-form-action="transfer"
        on:click={async event => {
          void 0
        }}
      >
        SUBMIT
      </Button>
    </Card.Footer>
  </Card.Root>
</form>

<style lang="postcss">
  .card-section-heading {
    @apply font-bold font-supermolot text-xl mt-2 mb-1;
  }
</style>
