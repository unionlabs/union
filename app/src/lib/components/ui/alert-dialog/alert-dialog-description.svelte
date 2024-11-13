<script lang="ts">
import { AlertDialog as AlertDialogPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = AlertDialogPrimitive.DescriptionProps

	interface Props {
		class?: $$Props["class"];
		children?: import('svelte').Snippet;
		[key: string]: any
	}

	let { class: className = undefined, children, ...rest }: Props = $props();

</script>

<AlertDialogPrimitive.Description
	class={cn("text-sm text-muted-foreground", className)}
	{...rest}
>
	{@render children?.()}
</AlertDialogPrimitive.Description>
