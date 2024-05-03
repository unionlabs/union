<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = HTMLAttributes<HTMLDivElement> & {
  el?: HTMLDivElement
}
export let el: $$Props["el"] = undefined
let className: $$Props["class"] = undefined
export { className as class }
</script>

<div
  bind:this={el}
  class={cn('grid gap-1.5 p-4 text-center sm:text-left', className)}
  {...$$restProps}
>
  <slot />
</div>
