<script lang="ts">
import { Drawer as DrawerPrimitive } from "vaul-svelte"
import { cn } from "$lib/utilities/shadcn.js"

let {
  ref = $bindable(null),
  class: className,
  ...restProps
}: DrawerPrimitive.OverlayProps = $props()
</script>

<DrawerPrimitive.Overlay
	bind:ref
	class={cn("fixed inset-0 z-50 bg-black/80", className)}
	{...restProps}
/>
