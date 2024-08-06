<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = HTMLAttributes<HTMLSpanElement>

let className: string | undefined | null = undefined
export { className as class }
</script>

<span
	class={cn("text-muted-foreground ml-auto text-xs tracking-widest", className)}
	{...$$restProps}
>
	<slot />
</span>
