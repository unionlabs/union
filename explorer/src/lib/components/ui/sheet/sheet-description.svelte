<script lang="ts">
import { cn } from "$lib/utils.js"
import { Dialog as SheetPrimitive } from "bits-ui"

let {
  ref = $bindable(null),
  class: className,
  ...restProps
}: SheetPrimitive.DescriptionProps = $props()
</script>

<SheetPrimitive.Description
  bind:ref
  data-slot="sheet-description"
  class={cn("text-muted-foreground text-sm", className)}
  {...restProps}
/>
