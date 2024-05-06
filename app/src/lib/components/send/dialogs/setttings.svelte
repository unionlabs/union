<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import Search from "virtual:icons/lucide/search"
import * as Dialog from "$lib/components/ui/dialog"
import { Input } from "$lib/components/ui/input/index.js"
import { Button } from "$lib/components/ui/button/index.js"

export let dialogOpen = false
</script>

<Dialog.Root
  closeOnEscape={true}
  preventScroll={true}
  bind:open={dialogOpen}
  closeOnOutsideClick={true}
>
  <Dialog.Content
    class={cn([
      'border-solid border-accent overflow-auto flex flex-col items-start rounded-md',
      'max-w-[90%] sm:max-w-[375px] border-[1px] pt-4 pb-1 px-2',
    ])}
  >
    <Dialog.Header class="max-h-min h-16 p-2 w-full space-y-3">
      <Dialog.Title class="font-extrabold text-2xl text-center -mt-2">Preference</Dialog.Title>
    </Dialog.Header>
    <Dialog.Description class="size-full">
     
    </Dialog.Description>
  </Dialog.Content>
</Dialog.Root>
