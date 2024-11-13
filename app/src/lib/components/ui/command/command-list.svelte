<script lang="ts">
import { Command as CommandPrimitive } from "cmdk-sv"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = CommandPrimitive.ListProps
	interface Props {
		class?: string | undefined | null;
		children?: import('svelte').Snippet;
		[key: string]: any
	}

	let { class: className = undefined, children, ...rest }: Props = $props();

</script>

<CommandPrimitive.List
	class={cn("max-h-[300px] overflow-y-auto overflow-x-hidden", className)}
	{...rest}
>
	{@render children?.()}
</CommandPrimitive.List>
