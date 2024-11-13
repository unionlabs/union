<script lang="ts">
import { Command as CommandPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

let {
  ref = $bindable(null),
  class: className,
  children,
  heading,
  ...restProps
}: CommandPrimitive.GroupProps & {
  heading?: string
} = $props()
</script>

<CommandPrimitive.Group
	class={cn("text-foreground overflow-hidden p-1", className)}
	bind:ref
	{...restProps}
>
	{#if heading}
		<CommandPrimitive.GroupHeading
			class="text-muted-foreground px-2 py-1.5 text-xs font-medium"
		>
			{heading}
		</CommandPrimitive.GroupHeading>
	{/if}
	<CommandPrimitive.GroupItems {children} />
</CommandPrimitive.Group>
