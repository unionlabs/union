<script lang="ts">
import { Command as CommandPrimitive } from "cmdk-sv"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = CommandPrimitive.SeparatorProps
	interface Props {
		class?: string | undefined | null;
		[key: string]: any
	}

	let { class: className = undefined, ...rest }: Props = $props();

</script>

<CommandPrimitive.Separator class={cn("bg-border -mx-1 h-px", className)} {...rest} />
