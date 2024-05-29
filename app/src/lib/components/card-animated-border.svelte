<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"

export let width = "w-48"
export let height = "h-48"
export let animationSpeed = 6
export let textSize = "text-sm"
</script>

<div
  class={cn(
    width,
    height,
    'relative overflow-hidden rounded-xl border border-gray-800 p-[1px] backdrop-blur-3xl',
  )}
>
  <span
    class={cn(
      `animate-[spin_${animationSpeed}s_linear_infinite]`,
      'absolute inset-[-1000%] bg-[conic-gradient(from_90deg_at_50%_50%,#E2CBFF_0%,#393BB2_50%,#E2CBFF_100%)]',
    )}
  />

  <div
    class={cn(
      textSize,
      'text-white inline-flex size-full items-center justify-center rounded-xl bg-background px-3 py-1 font-medium backdrop-blur-3xl',
    )}
  >
    <slot />
  </div>
</div>
