<script lang="ts">
import { Pagination as PaginationPrimitive } from "bits-ui"
import ChevronRight from "lucide-svelte/icons/chevron-right"
import { buttonVariants } from "#/components/svelte/ui/button/index.js"
import { cn } from "#/lib/shadcn.js"

let {
  ref = $bindable(null),
  class: className,
  children,
  ...restProps
}: PaginationPrimitive.NextButtonProps = $props()
</script>

{#snippet Fallback()}
  <span>Next</span>
  <ChevronRight class="size-4" />
{/snippet}

<PaginationPrimitive.NextButton
  bind:ref
  class={cn(
    buttonVariants({
      variant: 'ghost',
      class:
        'gap-1 pr-2.5 bg-transparent border-[1px] border-accent-500 hover:cursor-pointer disabled:text-white/70 disabled:opacity-60',
    }),
    className,
  )}
  children={children || Fallback}
  {...restProps}
/>
