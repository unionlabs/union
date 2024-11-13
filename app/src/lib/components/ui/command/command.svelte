<script lang="ts">
import { Command as CommandPrimitive } from "cmdk-sv"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = CommandPrimitive.CommandProps


	interface Props {
		value?: $$Props["value"];
		class?: string | undefined | null;
		children?: import('svelte').Snippet;
		[key: string]: any
	}

	let { value = $bindable(undefined), class: className = undefined, children, ...rest }: Props = $props();

</script>

<CommandPrimitive.Root
	class={cn(
		"bg-popover text-popover-foreground flex h-full w-full flex-col overflow-hidden",
		className
	)}
	bind:value
	{...rest}
>
	{@render children?.()}
</CommandPrimitive.Root>
