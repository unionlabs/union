<script lang="ts">
import { Avatar as AvatarPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = AvatarPrimitive.ImageProps

	interface Props {
		class?: $$Props["class"];
		src?: $$Props["src"];
		alt?: $$Props["alt"];
		[key: string]: any
	}

	let { class: className = undefined, src = undefined, alt = undefined, ...rest }: Props = $props();

</script>

<AvatarPrimitive.Image
	{src}
	{alt}
	class={cn("aspect-square h-full w-full", className)}
	{...rest}
/>
