<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import * as Dialog from "$lib/components/ui/dialog"
import { Button } from "$lib/components/ui/button/index.js"
import { Badge } from "$lib/components/ui/badge"
import type { UserAddresses } from "$lib/types.ts"
import { toast } from "svelte-sonner"

export let kind: "from" | "to"
export let dialogOpen = false
export let onChainSelect: (newSelectedChain: string) => void
export let chains: Array<{ chain_id: string; display_name: string; rpc_type: string }>
export let selectedChain: string
export let userAddr: UserAddresses | null

$: document.body.style.overflow = dialogOpen ? "hidden" : "auto"

function selectChain(chain: { chain_id: string; display_name: string; rpc_type: string }) {
  if (chain.rpc_type === "evm" && !userAddr?.evm) {
    toast.info(`Connect EVM wallet`)
    return
  }

  if (chain.rpc_type === "cosmos" && !userAddr?.cosmos) {
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
  <Dialog.Content
    class={cn(
      'border-solid overflow-auto flex flex-col items-start p-0 pt-4 pb-2',
    )}
  >
    <Dialog.Header class="max-h-min p-2 w-full">
      <Dialog.Title class="px-2">
        Select {kind} Network
      </Dialog.Title>
    </Dialog.Header>
    <Dialog.Description class="size-full">
      <!--
      <div class="relative mr-auto flex-1 w-full">
        <Search class="absolute left-2.5 top-2.5 size-4 text-muted-foreground" />
        <Input
          type="search"
          pattern="[a-z]"
          autocorrect="off"
          spellcheck="false"
          autocapitalize="off"
          placeholder="choose origin chain"
          on:input={event => handleChainSearch(event)}
          class="w-full bg-current/95 pl-8 self-stretch lowercase focus-visible:ring-0 rounded-none focus-visible:outline-none border-x-0"
        />
      </div>
      !-->
      <ul class="flex flex-col">
        {#each chains as chain, index}
          {@const selected = selectedChain === chain.chain_id}
          <li
            class={cn(
              'dark:text-accent-foreground flex flex-col',
            )}
          >
            <Button
              variant={'ghost'}
              on:click={() => {selectChain(chain)}}
              class={cn('size-full px-4 py-2 w-full text-foreground rounded-none flex items-center justify-between dark:hover:text-black',
                selected ? 'bg-muted-foreground text-background' : ''
              )}
            >
              <span class="text-lg font-bold">
                {chain.display_name}
              </span>
                {#if (chain.rpc_type === "evm" && !userAddr?.evm) || (chain.rpc_type === "cosmos" && !userAddr?.cosmos) }
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
