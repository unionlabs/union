<script lang="ts">
import type { WithElementRef } from "bits-ui"
import type { HTMLLiAttributes } from "svelte/elements"

let {
  ref = $bindable(null),
  children,
  ...restProps
}: WithElementRef<
  HTMLLiAttributes & {
    isVisible?: boolean
  }
> = $props()
</script>

<li
  bind:this={ref}
  {...restProps}
>
  {@render children?.()}
</li>
