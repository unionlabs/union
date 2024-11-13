<script lang="ts">
  import { createBubbler } from 'svelte/legacy';

  const bubble = createBubbler();
import type { HTMLAttributes } from "svelte/elements"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = HTMLAttributes<HTMLTableSectionElement>

  interface Props {
    class?: $$Props["class"];
    children?: import('svelte').Snippet;
    [key: string]: any
  }

  let { class: className = undefined, children, ...rest }: Props = $props();

</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<thead class={cn('[&_tr]:border-b', className)} {...rest} onclick={bubble('click')} onkeydown={bubble('keydown')}>
  {@render children?.()}
</thead>
