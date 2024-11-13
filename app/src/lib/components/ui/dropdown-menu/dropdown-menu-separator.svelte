<script lang="ts">
import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = DropdownMenuPrimitive.SeparatorProps

	interface Props {
		class?: $$Props["class"];
		[key: string]: any
	}

	let { class: className = undefined, ...rest }: Props = $props();

</script>

<DropdownMenuPrimitive.Separator
	class={cn("-mx-1 my-1 h-px bg-muted", className)}
	{...rest}
/>
