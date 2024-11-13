<script lang="ts">
import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui"

type $$Props = DropdownMenuPrimitive.RadioGroupProps

	interface Props {
		value?: $$Props["value"];
		children?: import('svelte').Snippet;
		[key: string]: any
	}

	let { value = $bindable(undefined), children, ...rest }: Props = $props();
</script>

<DropdownMenuPrimitive.RadioGroup {...rest} bind:value>
	{@render children?.()}
</DropdownMenuPrimitive.RadioGroup>
