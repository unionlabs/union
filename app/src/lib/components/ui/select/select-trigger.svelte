<script lang="ts">
import { Select as SelectPrimitive } from "bits-ui"
import ChevronDown from "virtual:icons/lucide/chevron-down"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = SelectPrimitive.TriggerProps
type $$Events = SelectPrimitive.TriggerEvents

  interface Props {
    class?: $$Props["class"];
    children?: import('svelte').Snippet<[any]>;
    [key: string]: any
  }

  let { class: className = undefined, children, ...rest }: Props = $props();


  const children_render = $derived(children);
</script>

<SelectPrimitive.Trigger
  class={cn(
    'flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1',
    className,
  )}
  {...rest}
  
  on:click
  on:keydown
>
  {#snippet children({ builder })}
    {@render children_render?.({ builder, })}
    <div>
      <ChevronDown class="h-4 w-4 opacity-50" />
    </div>
  {/snippet}
</SelectPrimitive.Trigger>
