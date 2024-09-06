<script lang="ts">
import { toast } from "svelte-sonner"
import { cn } from "$lib/utilities/shadcn.ts"
import { Badge } from "$lib/components/ui/badge"
import type { UserAddresses } from "$lib/types.ts"
import * as Dialog from "$lib/components/ui/dialog"
import { Button } from "$lib/components/ui/button/index.js"

export let kind: "from" | "to"
export let dialogOpen = false
export let onChainSelect: (newSelectedChain: string) => void
export let chains: Array<{ chain_id: string; display_name: string; rpc_type: string }>
export let selectedChain: string
export let userAddress: UserAddresses | null

$: document.body.style.overflow = dialogOpen ? "hidden" : "auto"

function selectChain(chain: { chain_id: string; display_name: string; rpc_type: string }) {
  if (chain.rpc_type === "evm" && !userAddress?.evm) {
    toast.info(`Connect EVM wallet`)
    return
  }

  if (chain.rpc_type === "cosmos" && !userAddress?.cosmos) {
    toast.info(`Connect Cosmos wallet`)
    return
  }
  onChainSelect(chain.chain_id)
  dialogOpen = false
}
</script>

<Dialog.Root
  closeOnEscape={true}
  preventScroll={true}
  bind:open={dialogOpen}
  closeOnOutsideClick={true}
>
  <Dialog.Content class={cn('border-solid overflow-auto flex flex-col items-start p-0 pt-4 pb-2')}>
    <Dialog.Header class="max-h-min p-2 w-full">
      <Dialog.Title class="px-2">
        Select {kind} Network
      </Dialog.Title>
    </Dialog.Header>
    <Dialog.Description class="size-full">
      <ul class="flex flex-col">
        {#each chains as chain, index}
          {@const selected = selectedChain === chain.chain_id}
          <li class={cn('dark:text-accent-foreground flex flex-col')}>
            <Button
              variant={'ghost'}
              on:click={() => {
                selectChain(chain)
              }}
              class={cn(
                'size-full px-4 py-2 w-full text-foreground rounded-none flex items-center justify-between dark:hover:text-black',
                selected ? 'bg-muted-foreground text-background' : '',
              )}
            >
              <span class="text-lg font-bold">
                {chain.display_name}
              </span>
              {#if (chain.rpc_type === 'evm' && !userAddress?.evm) || (chain.rpc_type === 'cosmos' && !userAddress?.cosmos)}
                <Badge variant={selected ? 'secondary' : 'default'}>Disconnected</Badge>
              {:else}
                <Badge variant={selected ? 'secondary' : 'default'}>Connected</Badge>
              {/if}
            </Button>
          </li>
        {/each}
      </ul>
    </Dialog.Description>
  </Dialog.Content>
</Dialog.Root>
