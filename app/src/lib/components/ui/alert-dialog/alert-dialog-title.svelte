<script lang="ts">
import { AlertDialog as AlertDialogPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = AlertDialogPrimitive.TitleProps

	interface Props {
		class?: $$Props["class"];
		level?: $$Props["level"];
		children?: import('svelte').Snippet;
		[key: string]: any
	}

	let { class: className = undefined, level = "h3", children, ...rest }: Props = $props();

</script>

<AlertDialogPrimitive.Title class={cn("text-lg font-semibold", className)} {level} {...rest}>
	{@render children?.()}
</AlertDialogPrimitive.Title>
