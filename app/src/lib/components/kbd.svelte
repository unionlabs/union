<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"

  interface Props {
    class: string;
    children?: import('svelte').Snippet;
  }

  let { class: className, children }: Props = $props();


</script>

<kbd
  class={cn(
    "h-6 px-1.5 gap-1",
    "bg-muted pointer-events-none select-none rounded border font-mono text-[12px] font-medium opacity-100",
    className
  )}
>
  {@render children?.()}
</kbd>

<style lang="postcss"></style>
