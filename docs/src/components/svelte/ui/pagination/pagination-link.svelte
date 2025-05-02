<script lang="ts">
import { buttonVariants, type Props } from "#/components/svelte/ui/button/index.js"
import { cn } from "#/lib/shadcn.js"
import { Pagination as PaginationPrimitive } from "bits-ui"

let {
  ref = $bindable(null),
  class: className,
  size = "icon",
  isActive = false,
  page,
  children,
  ...restProps
}:
  & PaginationPrimitive.PageProps
  & Props
  & {
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
      variant: isActive ? "outline" : "ghost",
      size,
      class: isActive && "text-black",
    }),
    className,
  )}
  children={children || Fallback}
  {...restProps}
/>
