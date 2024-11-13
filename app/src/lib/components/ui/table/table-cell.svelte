<script lang="ts">
  import { createBubbler } from 'svelte/legacy';

  const bubble = createBubbler();
import type { HTMLTdAttributes } from "svelte/elements"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = HTMLTdAttributes

  interface Props {
    class?: $$Props["class"];
    children?: import('svelte').Snippet;
    [key: string]: any
  }

  let { class: className = undefined, children, ...rest }: Props = $props();

</script>

<td
  class={cn('px-2 py-1 align-middle [&:has([role=checkbox])]:pr-0', className)}
  {...rest}
  onclick={bubble('click')}
  onkeydown={bubble('keydown')}
>
  {@render children?.()}
</td>
