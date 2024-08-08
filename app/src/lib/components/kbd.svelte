<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"

export let className: string

export { className as class }
</script>

<kbd
  class={cn(
    "h-6 px-1.5 gap-1",
    "bg-muted pointer-events-none select-none rounded border font-mono text-[12px] font-medium opacity-100",
    className
  )}
>
  <slot />
</kbd>

<style lang="postcss"></style>
