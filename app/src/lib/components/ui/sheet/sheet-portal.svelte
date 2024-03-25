<script lang="ts">
import { Dialog as SheetPrimitive } from "bits-ui"
import { cn } from "$/lib/utilities/shadcn.js"

type $$Props = SheetPrimitive.PortalProps

let className: $$Props["class"] = undefined
export { className as class }
</script>

<SheetPrimitive.Portal class={cn(className)} {...$$restProps}>
  <slot />
</SheetPrimitive.Portal>
