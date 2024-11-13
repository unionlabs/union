<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import type { HeadingLevel } from "./index.js"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = HTMLAttributes<HTMLHeadingElement> & {
  tag?: HeadingLevel
}

  interface Props {
    class?: $$Props["class"];
    tag?: $$Props["tag"];
    children?: import('svelte').Snippet;
    [key: string]: any
  }

  let { class: className = undefined, tag = "h3", children, ...rest }: Props = $props();

</script>

<svelte:element
  this={tag}
  class={cn('text-xl font-supermolot font-bold uppercase', className)}
  {...rest}
>
  {@render children?.()}
</svelte:element>
