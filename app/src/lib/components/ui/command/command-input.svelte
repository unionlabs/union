<script lang="ts">
import { Command as CommandPrimitive } from "bits-ui"
import Search from "lucide-svelte/icons/search"
import { cn } from "$lib/utilities/shadcn.js"

let {
  ref = $bindable(null),
  class: className,
  value = $bindable(""),
  ...restProps
}: CommandPrimitive.InputProps = $props()
</script>

<div class="flex items-center border-b px-2" data-command-input-wrapper="">
	<Search class="mr-2 size-4 shrink-0 opacity-50" />
	<CommandPrimitive.Input
		class={cn(
			"placeholder:text-muted-foreground flex h-11 w-full rounded-md bg-transparent py-3 text-sm outline-none disabled:cursor-not-allowed disabled:opacity-50",
			className
		)}
		bind:ref
		{...restProps}
		bind:value
	/>
</div>
