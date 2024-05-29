<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import Search from "virtual:icons/lucide/search"
import * as Dialog from "$lib/components/ui/dialog"
import { Input } from "$lib/components/ui/input/index.js"
import { Button } from "$lib/components/ui/button/index.js"

export let kind: "from" | "to"
export let dialogOpen = false
export let chainSearchResults: Array<{ name: string; id: string; icon: string; live: boolean }>
export let handleChainSearch: (event: InputEvent) => void
export let handleChainSelect: (name: string, target: "fromChain" | "toChain") => void
export let queryParams: any

$: {
  console.log(queryParams)
}
$: {
  if (dialogOpen) document.body.style.overflow = "hidden"
  else document.body.style.overflow = "auto"
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
      'border-solid border-accent overflow-auto flex flex-col items-start rounded-md',
      'max-w-[90%] sm:max-w-[375px] border-[1px] pt-4 pb-1 px-0',
    )}
  >
    <Dialog.Header class="max-h-min p-2 w-full space-y-3">
      <Dialog.Title class="font-extrabold text-2xl text-center -mt-2">
        Select {kind} Network
      </Dialog.Title>
    </Dialog.Header>
    <Dialog.Description class="size-full">
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
      <ul class="my-3 space-y-1 px-2">
        {#each chainSearchResults as { name, id: chainId, icon, live }, index}
          <li
            class={cn(
              live ? 'cursor-pointer' : 'cursor-not-allowed',
              'pb-2 dark:text-accent-foreground flex flex-col h-full justify-start align-middle space-x-3.5',
            )}
          >
            <Button
              disabled={!live}
              variant={queryParams.from === name.toLowerCase() ? 'secondary' : 'ghost'}
              on:click={() => handleChainSelect(chainId, kind === 'from' ? 'fromChain' : 'toChain')}
              class={cn('w-full flex justify-start space-x-4 p-2 pl-3 h-[55px] my-auto rounded-sm')}
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
