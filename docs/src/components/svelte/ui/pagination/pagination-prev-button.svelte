<script lang="ts">
import { Pagination as PaginationPrimitive } from "bits-ui"
import ChevronLeft from "lucide-svelte/icons/chevron-left"
import { buttonVariants } from "#/components/svelte/ui/button/index.js"
import { cn } from "#/lib/shadcn.js"

let {
  ref = $bindable(null),
  class: className,
  children,
  ...restProps
}: PaginationPrimitive.PrevButtonProps = $props()
</script>

{#snippet Fallback()}
  <ChevronLeft class="size-4" />
  <span>Previous</span>
{/snippet}

<PaginationPrimitive.PrevButton
  bind:ref
  class={cn(
    buttonVariants({
      variant: 'ghost',
      class:
        'gap-1 pl-2.5 bg-transparent border-[1px] border-accent-500 hover:cursor-pointer disabled:text-gray-400 disabled:opacity-60',
    }),
    className,
  )}
  children={children || Fallback}
  {...restProps}
/>
