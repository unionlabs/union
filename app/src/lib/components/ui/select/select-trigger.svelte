<script lang="ts">
import { Select as SelectPrimitive, type WithoutChild } from "bits-ui"
import ChevronDown from "lucide-svelte/icons/chevron-down"
import { cn } from "$lib/utilities/shadcn.js"

let {
  ref = $bindable(null),
  class: className,
  children,
  ...restProps
}: WithoutChild<SelectPrimitive.TriggerProps> = $props()
</script>

<SelectPrimitive.Trigger
	bind:ref
	class={cn(
		"border-input bg-background ring-offset-background data-[placeholder]:text-muted-foreground focus:ring-ring flex h-10 w-full items-center justify-between rounded-md border px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1",
		className
	)}
	{...restProps}
>
	{@render children?.()}
	<ChevronDown class="size-4 opacity-50" />
</SelectPrimitive.Trigger>
