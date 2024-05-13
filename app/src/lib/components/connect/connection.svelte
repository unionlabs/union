<script lang="ts">
import clsx from "clsx"
import type { Props } from "./index.ts"
import { copy } from "@svelte-put/copy"
import XIcon from "lucide-svelte/icons/x"
import CopyIcon from "lucide-svelte/icons/copy"
import CheckIcon from "lucide-svelte/icons/check"
import { Button } from "$/lib/components/ui/button"
import LoaderCircleIcon from "lucide-svelte/icons/loader-circle"
import { truncateEvmAddress, truncateUnionAddress } from "$/lib/wallet/utilities/format.ts"

export let chain: "cosmos" | "evm"
type T = $$Generic<typeof chain>

type $$Props = Props<T>

export let address: $$Props["address"]
export let hoverState: $$Props["hoverState"]
export let connectStatus: $$Props["connectStatus"]
export let onConnectClick: $$Props["onConnectClick"]
export let onDisconnectClick: $$Props["onDisconnectClick"]
export let connectedWalletName: $$Props["connectedWalletName"]
export let chainWalletsInformation: $$Props["chainWalletsInformation"]

$: connectText =
  connectStatus === "connected" && address && address?.length > 0
    ? chain === "evm"
      ? truncateEvmAddress(address, -1)
      : truncateUnionAddress(address, -1)
    : chain === "evm"
      ? "Sepolia"
      : "Union"

let copyClicked = false
const toggleCopy = () => (copyClicked = !copyClicked)
const onCopyClick = () => [toggleCopy(), setTimeout(() => toggleCopy(), 1_500)]
</script>

<Button
  tabindex={0}
  {...$$restProps}
  id={`${chain}-connect`}
  on:click={_event => onCopyClick()}
  builders={[{ action: node => copy(node, { text: address }) }]}
  variant={connectStatus === 'connected' ? 'default' : 'ghost'}
  class={clsx(
    ['px-2 w-full focus:ring-0 ring-transparent focus-visible:ring-0 flex justify-start'],
    connectStatus === 'disconnected' &&
      'hover:bg-transparent !text-white pointer-events-none text-md font-bold',
  )}
>
  <span
    class={clsx([
      'text-[12.5px] w-full sm:text-sm text-left',
      { 'text-lg sm:text-lg': connectText === 'Sepolia' || connectText === 'Union' },
    ])}
  >
    {connectText}
  </span>
  {#if connectStatus === 'connected' && address?.length}
    {#if copyClicked}
      <CheckIcon class="size-4 ml-auto" />
    {:else}
      <CopyIcon class="size-4 ml-auto text-black/70" />
    {/if}
  {/if}
</Button>

<div class="mt-2 mx-auto flex items-center flex-col">
  {#each chainWalletsInformation as { name, id, icon, download }, index (index)}
    {@const walletIdentifier = chain === 'evm' ? id : name}
    <div
      role="row"
      tabindex={0}
      data-index={index}
      on:mouseleave={() => (hoverState = connectedWalletName === name ? 'none' : 'none')}
      on:mouseenter={() => (hoverState = connectedWalletName === name ? 'hover' : 'none')}
      class={clsx([
        'flex flex-col w-full justify-start mb-3',
        {
          'animate-pulse animation-delay-75':
            connectStatus === 'connecting' || connectStatus === 'reconnecting',
          hidden: connectStatus === 'connected' && connectedWalletName !== name,
          block: connectStatus === 'disconnected' || connectStatus === 'connected',
        },
      ])}
    >
      <Button
        type="button"
        variant="outline"
        disabled={connectStatus === 'connected' &&
          (['connecting', 'reconnecting'].includes(connectStatus) || connectedWalletName !== name)}
        class={clsx([
          'capitalize justify-start h-12 text-lg ring-0 focus:ring-0 ring-transparent',
          {
            'opacity-60 hover:opacity-100':
              connectStatus === 'disconnected' || connectStatus == undefined,
            'border-[#037791]': connectStatus === 'connected' && connectedWalletName === name,
            'hover:text-rose-50 border-rose-900 hover:bg-transparent':
              hoverState === 'hover' && connectedWalletName === name,
          },
        ])}
        on:click={() =>
          connectStatus === 'connected' ? onDisconnectClick() : onConnectClick(walletIdentifier)}
      >
        <img src={icon} alt={name} class="size-7 mr-3 text-white" />
        {name}
        {#if connectStatus === 'connected'}
          {#if connectedWalletName === name}
            {#if hoverState === 'hover'}
              <XIcon class="ml-auto" />
            {:else}
              <CheckIcon class="ml-auto" />
            {/if}
          {/if}
        {:else if connectStatus === 'connecting' || connectStatus === 'reconnecting'}
          {#if connectedWalletName === name}
            <LoaderCircleIcon class="animate-spin ml-auto" />
          {:else}
            <LoaderCircleIcon class="animate-spin ml-auto opacity-0" />
          {/if}
        {/if}
      </Button>
    </div>
  {/each}
</div>
