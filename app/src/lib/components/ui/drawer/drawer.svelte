<script lang="ts">
import { Drawer as DrawerPrimitive } from "vaul-svelte"

type $$Props = DrawerPrimitive.Props
	interface Props {
		shouldScaleBackground?: $$Props["shouldScaleBackground"];
		open?: $$Props["open"];
		activeSnapPoint?: $$Props["activeSnapPoint"];
		children?: import('svelte').Snippet;
		[key: string]: any
	}

	let {
		shouldScaleBackground = true,
		open = $bindable(false),
		activeSnapPoint = $bindable(undefined),
		children,
		...rest
	}: Props = $props();
</script>

<DrawerPrimitive.Root {shouldScaleBackground} bind:open bind:activeSnapPoint {...rest}>
	{@render children?.()}
</DrawerPrimitive.Root>
