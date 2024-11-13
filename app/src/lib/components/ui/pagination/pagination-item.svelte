<script lang="ts">
import type { HTMLLiAttributes } from "svelte/elements"
import type { WithElementRef } from "bits-ui"

let { ref = $bindable(null), children, ...restProps }: WithElementRef<HTMLLiAttributes> = $props()
</script>

<li bind:this={ref} {...restProps}>
	{@render children?.()}
</li>
