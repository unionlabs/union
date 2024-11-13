<script lang="ts">
import { Avatar as AvatarPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = AvatarPrimitive.Props

	interface Props {
		class?: $$Props["class"];
		delayMs?: $$Props["delayMs"];
		children?: import('svelte').Snippet;
		[key: string]: any
	}

	let { class: className = undefined, delayMs = undefined, children, ...rest }: Props = $props();

</script>

<AvatarPrimitive.Root
	{delayMs}
	class={cn("relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full", className)}
	{...rest}
>
	{@render children?.()}
</AvatarPrimitive.Root>
