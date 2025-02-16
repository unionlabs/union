<script lang="ts">
import type { HTMLTdAttributes } from "svelte/elements"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = HTMLTdAttributes

let className: $$Props["class"] = undefined
export { className as class }
</script>

<td
  class={cn('px-[16px] py-[6px] align-middle [&:has([role=checkbox])]:pr-0', className)}
  {...$$restProps}
  on:click
  on:keydown
>
  <slot />
</td>
