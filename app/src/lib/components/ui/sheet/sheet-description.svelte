<script lang="ts">
import { Dialog as SheetPrimitive } from "bits-ui"
import { cn } from "$/lib/utilities/shadcn.js"

type $$Props = SheetPrimitive.DescriptionProps

let className: $$Props["class"] = undefined
export { className as class }
</script>

<SheetPrimitive.Description class={cn('text-sm text-muted-foreground', className)} {...$$restProps}>
  <slot />
</SheetPrimitive.Description>
