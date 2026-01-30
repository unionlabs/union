<script lang="ts" module>
	import { type VariantProps, tv } from "tailwind-variants";

	export const badgeVariants = tv({
		base: "inline-flex w-fit shrink-0 items-center justify-center gap-1 overflow-hidden border-transparent px-1.5 py-0.5 text-[10px] font-mono whitespace-nowrap [&>svg]:pointer-events-none [&>svg]:size-3",
		variants: {
			variant: {
				default: "bg-foreground/10 text-muted-foreground",
				secondary: "bg-muted text-muted-foreground",
				destructive: "bg-red-500/10 text-red-500",
				success: "bg-green-500/10 text-green-500",
				warning: "bg-amber-500/10 text-amber-500",
				info: "bg-blue-500/10 text-blue-500",
				purple: "bg-purple-500/10 text-purple-500",
				cyan: "bg-cyan-500/10 text-cyan-500",
				emerald: "bg-emerald-500/10 text-emerald-500",
				orange: "bg-orange-500/10 text-orange-500",
				pink: "bg-pink-500/10 text-pink-500",
				outline: "border border-border text-foreground",
			},
		},
		defaultVariants: {
			variant: "default",
		},
	});

	export type BadgeVariant = VariantProps<typeof badgeVariants>["variant"];
</script>

<script lang="ts">
	import type { HTMLAnchorAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		href,
		class: className,
		variant = "default",
		children,
		...restProps
	}: WithElementRef<HTMLAnchorAttributes> & {
		variant?: BadgeVariant;
	} = $props();
</script>

<svelte:element
	this={href ? "a" : "span"}
	bind:this={ref}
	data-slot="badge"
	{href}
	class={cn(badgeVariants({ variant }), className)}
	{...restProps}
>
	{@render children?.()}
</svelte:element>
