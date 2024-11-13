<script lang="ts">
import { Command as CommandPrimitive } from "cmdk-sv"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = CommandPrimitive.EmptyProps
	interface Props {
		class?: string | undefined | null;
		children?: import('svelte').Snippet;
		[key: string]: any
	}

	let { class: className = undefined, children, ...rest }: Props = $props();

</script>

<CommandPrimitive.Empty class={cn("py-6 text-center text-sm", className)} {...rest}>
	{@render children?.()}
</CommandPrimitive.Empty>
