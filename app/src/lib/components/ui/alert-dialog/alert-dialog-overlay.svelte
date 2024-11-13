<script lang="ts">
import { AlertDialog as AlertDialogPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

let {
  ref = $bindable(null),
  class: className,
  ...restProps
}: AlertDialogPrimitive.OverlayProps = $props()
</script>

<AlertDialogPrimitive.Overlay
	bind:ref
	class={cn(
		"data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0  fixed inset-0 z-50 bg-black/80",
		className
	)}
	{...restProps}
/>
