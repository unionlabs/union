<script lang="ts">
import { Checkbox as CheckboxPrimitive, type WithoutChildrenOrChild } from "bits-ui"
import Check from "lucide-svelte/icons/check"
import Minus from "lucide-svelte/icons/minus"
import { cn } from "$lib/utilities/shadcn.js"

let {
  ref = $bindable(null),
  checked = $bindable(false),
  indeterminate = $bindable(false),
  class: className,
  ...restProps
}: WithoutChildrenOrChild<CheckboxPrimitive.RootProps> = $props()
</script>

<CheckboxPrimitive.Root
	bind:ref
	class={cn(
		"border-primary ring-offset-background focus-visible:ring-ring data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground peer box-content size-4 shrink-0 rounded-sm border focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 data-[disabled=true]:cursor-not-allowed data-[disabled=true]:opacity-50",
		className
	)}
	bind:checked
	bind:indeterminate
	{...restProps}
>
	{#snippet children({ checked, indeterminate })}
		<div class="flex size-4 items-center justify-center text-current">
			{#if indeterminate}
				<Minus class="size-3.5" />
			{:else}
				<Check class={cn("size-3.5", !checked && "text-transparent")} />
			{/if}
		</div>
	{/snippet}
</CheckboxPrimitive.Root>
