<script lang="ts">
import type { HTMLThAttributes } from "svelte/elements"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = HTMLThAttributes

  interface Props {
    class?: $$Props["class"];
    children?: import('svelte').Snippet;
    [key: string]: any
  }

  let { class: className = undefined, children, ...rest }: Props = $props();

</script>

<th
  class={cn(
    'h-12 px-2 text-left align-middle font-bold uppercase [&:has([role=checkbox])]:pr-0',
    className,
  )}
  {...rest}
>
  {@render children?.()}
</th>
