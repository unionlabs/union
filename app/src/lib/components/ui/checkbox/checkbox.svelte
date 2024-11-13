<script lang="ts">
import { Checkbox as CheckboxPrimitive } from "bits-ui"
import Check from "virtual:icons/lucide/check"
import Minus from "virtual:icons/lucide/minus"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = CheckboxPrimitive.Props
type $$Events = CheckboxPrimitive.Events

  interface Props {
    class?: $$Props["class"];
    checked?: $$Props["checked"];
    [key: string]: any
  }

  let { class: className = undefined, checked = $bindable(false), ...rest }: Props = $props();

</script>

<CheckboxPrimitive.Root
  class={cn(
    'peer box-content h-4 w-4 shrink-0 rounded-sm border border-primary ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 data-[disabled=true]:cursor-not-allowed data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground data-[disabled=true]:opacity-50',
    className,
  )}
  bind:checked
  {...rest}
  on:click
>
  <CheckboxPrimitive.Indicator
    class={cn('flex h-4 w-4 items-center justify-center text-current')}
    
    
  >
    {#snippet children({ isChecked, isIndeterminate })}
        {#if isChecked}
        <Check class="h-3.5 w-3.5" />
      {:else if isIndeterminate}
        <Minus class="h-3.5 w-3.5" />
      {/if}
          {/snippet}
    </CheckboxPrimitive.Indicator>
</CheckboxPrimitive.Root>
