<script lang="ts">
import { Drawer as DrawerPrimitive } from "vaul-svelte"
import { cn } from "$lib/utilities/shadcn.js"

let { ref = $bindable(null), class: className, ...restProps }: DrawerPrimitive.TitleProps = $props()
</script>

<DrawerPrimitive.Title
	bind:ref
	class={cn("text-lg font-semibold leading-none tracking-tight", className)}
	{...restProps}
/>
