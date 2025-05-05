<script lang="ts">
import { cn } from "#/lib/shadcn.js"
import type { WithElementRef } from "bits-ui"
import type { HTMLAttributes } from "svelte/elements"

let {
  ref = $bindable(null),
  class: className,
  children,
  ...restProps
}: WithElementRef<HTMLAttributes<HTMLTableSectionElement>> = $props()
</script>

<thead
  bind:this={ref}
  class={cn("[&_tr]:border-b", className)}
  {...restProps}
>
  {@render children?.()}
</thead>
