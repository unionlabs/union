<script lang="ts">
import type { Props } from "./index.ts"
import XIcon from "virtual:icons/lucide/x"
import { cn } from "$lib/utilities/shadcn.ts"
import CopyIcon from "virtual:icons/lucide/copy"
import CheckIcon from "virtual:icons/lucide/check"
import { Button } from "$lib/components/ui/button"
import { copyTextAction } from "$lib/actions/copy.ts"
import LoaderCircleIcon from "virtual:icons/lucide/loader-circle"
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
  variant={connectStatus === 'connected' ? 'default' : 'ghost'}
  builders={[{ action: node => copyTextAction(node, { text: address }) }]}
  class={cn(
    'px-2 w-full focus:ring-0 ring-transparent focus-visible:ring-0 flex justify-start',
    connectStatus === 'disconnected' &&
      'hover:bg-transparent !text-white pointer-events-none text-md font-bold',
  )}
>
  <span
    class={cn(
      'w-full text-left font-mono',
      connectText === 'Sepolia' || connectText === 'Union'
        ? 'text-lg sm:text-sm'
        : 'text-sm sm:text-[15.5px]',
    )}
  >
    {connectText}
  </span>
  {#if connectStatus === 'connected' && address?.length}
    {#if copyClicked}
      <CheckIcon class="size-4 ml-auto" />
    {:else}
      <CopyIcon class="size-4 ml-auto dark:text-black/70" />
    {/if}
  {/if}
</Button>

<div class="mt-2 mx-auto flex items-center flex-col">
  {#each chainWalletsInformation as { name, id, icon, download }, index (index)}
    {@const walletIdentifier = id}
    <div
      role="row"
      tabindex={0}
      data-index={index}
      on:mouseleave={() => (hoverState = connectedWalletId === id ? 'none' : 'none')}
      on:mouseenter={() => (hoverState = connectedWalletId === id ? 'hover' : 'none')}
      class={cn(
        'flex',
        'flex-col w-full justify-start mb-3',
        connectStatus === 'connected' && connectedWalletId !== id ? 'hidden' : 'flex',
      )}
    >
      <Button
        type="button"
        variant="outline"
        class={cn(
          'capitalize justify-start h-12 text-lg ring-0 focus:ring-0 ring-transparent',
          connectStatus === 'connected' && connectedWalletId === id && 'border-[#037791]',
          (connectStatus === 'disconnected' || connectStatus == undefined) &&
            'opacity-60 hover:opacity-100',
          hoverState === 'hover' &&
            connectedWalletId === id &&
            'hover:text-rose-500 border-rose-900 hover:bg-transparent',
        )}
        on:click={() => {
          if (!walletIdentifier) return
          if (connectStatus === 'connected') onDisconnectClick()
          else onConnectClick(walletIdentifier)
        }}
      >
        <img src={icon} alt={name} class="size-7 mr-3 dark:text-white" />
        {name}
        {#if connectStatus === 'connected'}
          {#if connectedWalletId === id}
            {#if hoverState === 'hover'}
              <XIcon class="ml-auto" />
            {:else}
              <CheckIcon class="ml-auto" />
            {/if}
          {/if}
        {:else if connectStatus === 'connecting' || connectStatus === 'reconnecting'}
          {#if connectedWalletId === id}
            <LoaderCircleIcon class="animate-spin ml-auto" />
          {:else}
            <LoaderCircleIcon class="animate-spin ml-auto opacity-0" />
          {/if}
        {/if}
      </Button>
    </div>
  {/each}
</div>
