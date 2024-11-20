<script lang="ts">
import { Pagination as PaginationPrimitive } from "bits-ui"
import { cn } from "#/lib/shadcn.js"
import { type Props, buttonVariants } from "#/components/svelte/ui/button/index.js"

let {
  ref = $bindable(null),
  class: className,
  size = "icon",
  isActive = false,
  page,
  children,
  ...restProps
}: PaginationPrimitive.PageProps &
  Props & {
    isActive: boolean
  } = $props()
</script>

{#snippet Fallback()}
  {page.value}
{/snippet}

<PaginationPrimitive.Page
  bind:ref
  {page}
  class={cn(
    buttonVariants({
      variant: isActive ? 'outline' : 'ghost',
      size,
      class: isActive && 'text-black',
    }),
    className,
  )}
  children={children || Fallback}
  {...restProps}
/>
