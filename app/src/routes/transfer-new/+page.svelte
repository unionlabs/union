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
import { page } from "$app/stores"
import { toast } from "svelte-sonner"
import { goto } from "$app/navigation"
import { onDestroy, onMount } from "svelte"
import { cn } from "$lib/utilities/shadcn.ts"
import { userAddrEvm } from "$lib/wallet/evm"
import type { Step } from "$lib/stepper-types"
import { config } from "$lib/wallet/evm/config"
import { toIsoString } from "$lib/utilities/date"
import { truncate } from "$lib/utilities/format.ts"
import { userAddrCosmos } from "$lib/wallet/cosmos"
import Stepper from "$lib/components/stepper.svelte"
import { raise, sleep } from "$lib/utilities/index.ts"
import { userBalancesQuery } from "$lib/queries/balance"
import * as Card from "$lib/components/ui/card/index.ts"
import type { Chain, UserAddresses } from "$lib/types.ts"
import { Input } from "$lib/components/ui/input/index.js"
import { userAddrOnChain } from "$lib/utilities/address.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import ChainDialog from "./(component)/chain-dialog.svelte"
import Chevron from "../transfer/(components)/chevron.svelte"
import { getSupportedAsset, zip } from "$lib/utilities/helpers.ts"
import ArrowLeftRight from "virtual:icons/lucide/arrow-left-right"
import { getCosmosChainInfo } from "$lib/wallet/cosmos/chain-info.ts"
import ChainButton from "../transfer/(components)/chain-button.svelte"
import { submittedTransfers } from "$lib/stores/submitted-transfers.ts"
import AssetsDialog from "../transfer/(components)/assets-dialog.svelte"
import { parseUnits, formatUnits, type HttpTransport, getAddress } from "viem"
import { aptosStore, userAddressAptos, getAptosWallet } from "$lib/wallet/aptos"
import { cosmosStore, getCosmosOfflineSigner } from "$/lib/wallet/cosmos/config.ts"
import { type Writable, writable, derived, get, type Readable } from "svelte/store"
import { type TransferState, stepBefore, stepAfter } from "$lib/transfer/transfer.ts"
import CardSectionHeading from "../transfer/(components)/card-section-heading.svelte"
import { custom, switchChain, getConnectorClient, waitForTransactionReceipt } from "@wagmi/core"

type SearchParams = { [key: string]: string }

const querClient = useQueryClient()

function queryData<T extends Array<unknown>>(
  key: Array<string>,
  filter?: (value: T[number]) => boolean
): T {
  const data = querClient.getQueryData<T>(key) ?? []
  return (filter ? data.filter(filter) : data) as T
}

let chains = queryData<Array<Chain>>(["chains"], chain => chain.enabled_staging)

let transferQueryOptions = queryOptions<SearchParams>({
  enabled: query => false,
  placeholderData: keepPreviousData,
  queryKey: ["transfer-state", $page.url.toString()],
  initialData: Object.fromEntries($page.url.searchParams),
  queryFn: ({ queryKey, signal, meta }) => Object.fromEntries($page.url.searchParams)
})

let state = createQuery(transferQueryOptions)

const observer = new QueryObserver<SearchParams>(querClient, {
  enabled: query => false,
  queryKey: ["transfer-state"]
})

let userAddress = derived(
  [userAddrCosmos, userAddrEvm, userAddressAptos],
  ([$cosmos, $evm, $aptos]) => ({ evm: $evm, aptos: $aptos, cosmos: $cosmos })
)

$: asset = $page.url.searchParams.get("asset") || $state.data?.asset
$: receiver = $page.url.searchParams.get("receiver") || $state.data?.receiver
$: toChainId = $page.url.searchParams.get("toChainId") || $state.data?.toChainId
$: fromChainId = $page.url.searchParams.get("fromChainId") || $state.data?.fromChainId

$: toChain = writable(chains.find(chain => chain.chain_id === toChainId))
$: fromChain = writable(chains.find(chain => chain.chain_id === fromChainId))

$: rawBalances = userBalancesQuery({
  chains,
  connected: true,
  userAddr: $userAddress
})

$: balances = derived([rawBalances, fromChain], ([$rawBalances, $fromChain]) => {
  if (!($fromChain && $rawBalances)) return []
  const balances = $rawBalances.flatMap(x => x.data as any)
  const chainAssets = Object.groupBy($fromChain.assets, asset => asset.denom)
  return balances.map(balance => {
    try {
      const asset = chainAssets[balance.address]?.at(0)
      if (asset?.denom === balance?.address) return { ...balance, ...asset }
      return balance
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : error
      console.error(errorMessage)
      return balance
    }
  })
})

$: assetInfo = $balances.find(x => x?.address === asset)

/**
 * observer observs the transfer state and updates the url accordingly
 */
const unsubscribe = observer.subscribe(result => {
  if (!result.data) return
  const entry = Object.entries(result.data).at(-1)
  const [key, value] = entry ?? []
  if (!key) return
  console.info(key, value)
  const searchParams = new URLSearchParams({ ...$page.url.searchParams, ...result.data })
  if (value?.length === 0) searchParams.delete(key)
  goto(`${$page.url.pathname}?${searchParams.toString()}`, {
    keepFocus: true,
    replaceState: true
  })

  $state.refetch()
})

const updateParams = (kv: { [key: string]: string }) => {
  querClient.setQueryData<SearchParams>(["transfer-state"], _ => ({ ..._, ...kv }))
}

function swapChainsClick(_event: MouseEvent) {
  if (!(fromChainId && toChainId)) return
  updateParams({ fromChainId: toChainId, toChainId: fromChainId, asset: "" })
}

let [dialogOpenFromChain, dialogOpenToChain, dialogOpenAsset] = [false, false, false]

onMount(() => {
  // console.info($balances)
})

onDestroy(() => unsubscribe())
</script>

<Card.Root class={cn('w-[500px]')}>
  <Card.Header>header</Card.Header>
  <Card.Content class={cn('flex flex-col gap-4')}>
    <section>
      <CardSectionHeading>From</CardSectionHeading>
      <ChainButton bind:dialogOpen={dialogOpenFromChain}>
        {fromChainId ?? 'Select chain'}
      </ChainButton>
      <div class="flex flex-col items-center pt-4 -mb-6">
        <Button on:click={swapChainsClick} size="icon" variant="outline">
          <ArrowLeftRight class="size-5 dark:text-white rotate-90" />
        </Button>
      </div>
      <CardSectionHeading>To</CardSectionHeading>
      <ChainButton bind:dialogOpen={dialogOpenToChain} disabled={!$fromChain?.chain_id}>
        {toChainId ?? 'Select chain'}
      </ChainButton>
    </section>
    <Button class="w-full" variant="outline" on:click={() => (dialogOpenAsset = !dialogOpenAsset)}>
      <div class="flex-1 text-left font-bold text-md">
        {assetInfo?.display_symbol ?? 'Select Asset'}
      </div>
      <Chevron />
    </Button>
    receiver
    <Input name="from" bind:value={receiver} />
  </Card.Content>
  <Card.Footer class="flex flex-col gap-4 items-start">
    <Button
      disabled={false}
      on:click={async event => {
        console.info('submit')
      }}
      type="button"
    >
      SUBMIT
    </Button>
  </Card.Footer>
</Card.Root>

<ChainDialog
  {chains}
  kind="from"
  userAddress={$userAddress}
  selectedChain={fromChainId}
  dialogOpen={dialogOpenFromChain}
  onChainSelect={value => updateParams({ fromChainId: value })}
/>

{#if fromChainId}
  <ChainDialog
    {chains}
    kind="to"
    selectedChain={toChainId}
    userAddress={$userAddress}
    dialogOpen={dialogOpenToChain}
    onChainSelect={value => {
      // set receiver to self initially
      const destinationAddress = $userAddress[$toChain?.rpc_type]?.canonical
      if (!destinationAddress?.length) return updateParams({ toChainId: value })
      return updateParams({ toChainId: value, receiver: destinationAddress })
    }}
  />
{/if}

<AssetsDialog
  chain={$fromChain}
  assets={$balances}
  bind:dialogOpen={dialogOpenAsset}
  onAssetSelect={({ symbol, address }) => updateParams({ asset: address })}
/>

<style lang="postcss"></style>
