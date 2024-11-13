<script lang="ts">
import { Tabs as TabsPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

let { ref = $bindable(null), class: className, ...restProps }: TabsPrimitive.ContentProps = $props()
</script>

<TabsPrimitive.Content
	bind:ref
	class={cn(
		"ring-offset-background focus-visible:ring-ring mt-2 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2",
		className
	)}
	{...restProps}
/>
