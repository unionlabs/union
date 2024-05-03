<script lang="ts">
import type { Props } from "./index.ts"
import clsx from "clsx"
import { copy } from "@svelte-put/copy"
import XIcon from "lucide-svelte/icons/x"
import { cn } from "$lib/utilities/shadcn.ts"
import CopyIcon from "lucide-svelte/icons/copy"
import CheckIcon from "lucide-svelte/icons/check"
import { Button } from "$lib/components/ui/button"
import LoaderCircleIcon from "lucide-svelte/icons/loader-circle"
import { truncateEvmAddress, truncateUnionAddress } from "$lib/wallet/utilities/format.ts"

export let chain: "cosmos" | "evm"
type T = $$Generic<typeof chain>

type $$Props = Props<T>

export let address: $$Props["address"]
export let hoverState: $$Props["hoverState"]
export let connectStatus: $$Props["connectStatus"]
export let onConnectClick: $$Props["onConnectClick"]
export let onDisconnectClick: $$Props["onDisconnectClick"]
export let connectedWalletId: $$Props["connectedWalletId"]
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
  class={cn(
    ['px-2 w-full focus:ring-0 ring-transparent focus-visible:ring-0 flex justify-start'],
    connectStatus === 'disconnected' &&
      'hover:bg-transparent !text-white pointer-events-none text-md font-bold',
  )}
>
  <span
    class={cn([
      'w-full text-left',
      connectText === 'Sepolia' || connectText === 'Union' ? 'text-lg sm:text-sm' : 'text-[12.5px] sm:text-sm',
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
    {#if connectStatus === 'disconnected' || connectStatus === 'connected' || connectStatus === 'connecting' || connectStatus === 'reconnecting'}
      <div
        role="row"
        tabindex={0}
        data-index={index}
        on:mouseleave={() => (hoverState = connectedWalletId === name ? 'none' : 'none')}
        on:mouseenter={() => (hoverState = connectedWalletId === name ? 'hover' : 'none')}
        class={cn([
          'flex',
          'flex-col w-full justify-start mb-3',
          (connectStatus === 'connecting' || connectStatus === 'reconnecting') &&
            'animate-pulse animation-delay-75',
          connectStatus === 'connected' && [connectedWalletId !== name ? 'hidden' : ''],
        ])}
      >
        <Button
          type="button"
          variant="outline"
          disabled={connectStatus === 'connected' &&
            (['connecting', 'reconnecting'].includes(connectStatus) || connectedWalletId !== name)}
          class={cn([
            'capitalize justify-start h-12 text-lg ring-0 focus:ring-0 ring-transparent',
            (connectStatus === 'disconnected' || connectStatus == undefined) &&
              'opacity-60 hover:opacity-100',
            connectStatus === 'connected' && connectedWalletId === name && 'border-[#037791]',
            hoverState === 'hover' &&
              connectedWalletId === name &&
              'hover:text-rose-50 border-rose-900 hover:bg-transparent',
          ])}
          on:click={() => {
            if (!walletIdentifier) return
            if (connectStatus === 'connected') onDisconnectClick()
            else onConnectClick(walletIdentifier)
          }}
        >
          <img src={icon} alt={name} class="size-7 mr-3 text-white" />
          {name}
          {#if connectStatus === 'connected'}
            {#if connectedWalletId === name}
              {#if hoverState === 'hover'}
                <XIcon class="ml-auto" />
              {:else}
                <CheckIcon class="ml-auto" />
              {/if}
            {/if}
          {:else if connectStatus === 'connecting' || connectStatus === 'reconnecting'}
            {#if connectedWalletId === name}
              <LoaderCircleIcon class="animate-spin ml-auto" />
            {:else}
              <LoaderCircleIcon class="animate-spin ml-auto opacity-0" />
            {/if}
          {/if}
        </Button>
      </div>
    {/if}
  {/each}
</div>
