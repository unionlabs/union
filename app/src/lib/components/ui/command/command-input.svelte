<script lang="ts">
import { Command as CommandPrimitive } from "cmdk-sv"
import Search from "virtual:icons/lucide/search"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = CommandPrimitive.InputProps

let className: string | undefined | null = undefined
export { className as class }
export let value = ""
</script>

<div class="flex items-center border-b px-2" data-cmdk-input-wrapper="">
  <Search class="mr-2 h-4 w-4 shrink-0 opacity-50" />
  <CommandPrimitive.Input
    class={cn(
      'flex h-11 w-full rounded-md bg-transparent py-3 text-sm outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50',
      className,
    )}
    {...$$restProps}
    bind:value
  />
</div>
