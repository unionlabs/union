<script lang="ts">
import { AlertDialog as AlertDialogPrimitive } from "bits-ui"
import * as AlertDialog from "./index.js"
import { cn, flyAndScale } from "$lib/utilities/shadcn.js"

type $$Props = AlertDialogPrimitive.ContentProps


	interface Props {
		transition?: $$Props["transition"];
		transitionConfig?: $$Props["transitionConfig"];
		class?: $$Props["class"];
		children?: import('svelte').Snippet;
		[key: string]: any
	}

	let {
		transition = flyAndScale,
		transitionConfig = undefined,
		class: className = undefined,
		children,
		...rest
	}: Props = $props();

</script>

<AlertDialog.Portal>
	<AlertDialog.Overlay />
	<AlertDialogPrimitive.Content
		{transition}
		{transitionConfig}
		class={cn(
			"fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border bg-background p-6 shadow-lg sm:rounded-lg md:w-full",
			className
		)}
		{...rest}
	>
		{@render children?.()}
	</AlertDialogPrimitive.Content>
</AlertDialog.Portal>
