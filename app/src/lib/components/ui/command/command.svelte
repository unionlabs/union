<script lang="ts">
import { Command as CommandPrimitive } from "cmdk-sv"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = CommandPrimitive.CommandProps

export let value: $$Props["value"] = undefined

let className: string | undefined | null = undefined
export { className as class }
</script>

<CommandPrimitive.Root
  class={cn(
    className,
    'flex h-full flex-col overflow-hidden rounded-md bg-popover text-popover-foreground',
  )}
  bind:value
  {...$$restProps}
>
  <slot />
</CommandPrimitive.Root>
