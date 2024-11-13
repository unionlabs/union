<script lang="ts">
  import { run } from 'svelte/legacy';

import { toast } from "svelte-sonner"
import { cn } from "$lib/utilities/shadcn.ts"
import { Badge } from "$lib/components/ui/badge"
import * as Dialog from "$lib/components/ui/dialog"
import type { Chain, UserAddresses } from "$lib/types.ts"
import { Button } from "$lib/components/ui/button/index.js"

  interface Props {
    kind: "from" | "to";
    dialogOpen?: boolean;
    onChainSelect: (newSelectedChain: string) => void;
    chains: Array<Chain>;
    selectedChain: string;
    userAddress: UserAddresses | null;
  }

  let {
    kind,
    dialogOpen = $bindable(false),
    onChainSelect,
    chains,
    selectedChain,
    userAddress
  }: Props = $props();

run(() => {
    document.body.style.overflow = dialogOpen ? "hidden" : "auto"
  });

function selectChain(chain: Chain) {
  if (chain.rpc_type === "aptos" && !userAddress?.aptos) return toast.info(`Connect Aptos wallet`)

  if (chain.rpc_type === "evm" && !userAddress?.evm) return toast.info(`Connect EVM wallet`)

  if (chain.rpc_type === "cosmos" && !userAddress?.cosmos)
    return toast.info(`Connect Cosmos wallet`)

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
              {#if (chain.rpc_type === 'evm' && !userAddress?.evm) 
                || (chain.rpc_type === 'cosmos' && !userAddress?.cosmos)
                || (chain.rpc_type === 'aptos' && !userAddress?.aptos)}
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
