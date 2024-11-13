<script lang="ts">
import { Pagination as PaginationPrimitive } from "bits-ui"

import { cn } from "$lib/utilities/shadcn.js"

let {
  ref = $bindable(null),
  class: className,
  count = 0,
  perPage = 10,
  page = $bindable(1),
  siblingCount = 1,
  ...restProps
}: PaginationPrimitive.RootProps = $props()
</script>

<PaginationPrimitive.Root
	bind:ref
	class={cn("mx-auto flex w-full flex-col items-center", className)}
	{count}
	{perPage}
	{siblingCount}
	bind:page
	{...restProps}
/>
