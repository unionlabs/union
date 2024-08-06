<script lang="ts">
import { Avatar as AvatarPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = AvatarPrimitive.Props

let className: $$Props["class"] = undefined
export let delayMs: $$Props["delayMs"] = undefined
export { className as class }
</script>

<AvatarPrimitive.Root
	{delayMs}
	class={cn("relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full", className)}
	{...$$restProps}
>
	<slot />
</AvatarPrimitive.Root>
