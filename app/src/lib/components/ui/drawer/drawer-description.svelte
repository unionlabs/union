<script lang="ts">
import { Drawer as DrawerPrimitive } from "vaul-svelte"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = DrawerPrimitive.DescriptionProps

export let el: $$Props["el"] = undefined
let className: $$Props["class"] = undefined
export { className as class }
</script>

<DrawerPrimitive.Description
  bind:el
  class={cn('text-sm text-muted-foreground', className)}
  {...$$restProps}
>
  <slot />
</DrawerPrimitive.Description>
