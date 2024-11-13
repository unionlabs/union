<script lang="ts">
import { Command as CommandPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

let {
  ref = $bindable(null),
  class: className,
  ...restProps
}: CommandPrimitive.SeparatorProps = $props()
</script>

<CommandPrimitive.Separator class={cn("bg-border -mx-1 h-px", className)} bind:ref {...restProps} />
