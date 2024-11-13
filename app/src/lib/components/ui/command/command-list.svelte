<script lang="ts">
import { Command as CommandPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

let { ref = $bindable(null), class: className, ...restProps }: CommandPrimitive.ListProps = $props()
</script>

<CommandPrimitive.List
	class={cn("max-h-[300px] overflow-y-auto overflow-x-hidden", className)}
	{...restProps}
	bind:ref
/>
