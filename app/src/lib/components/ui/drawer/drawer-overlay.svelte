<script lang="ts">
import { Drawer as DrawerPrimitive } from "vaul-svelte"
import { cn } from "$/lib/utilities/shadcn.js"

type $$Props = DrawerPrimitive.OverlayProps

export let el: $$Props["el"] = undefined
let className: $$Props["class"] = undefined
export { className as class }
</script>

<DrawerPrimitive.Overlay
  bind:el
  class={cn('fixed inset-0 z-50 bg-black/80', className)}
  {...$$restProps}
>
  <slot />
</DrawerPrimitive.Overlay>
