<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = HTMLAttributes<HTMLDivElement>

  interface Props {
    class?: $$Props["class"];
    children?: import('svelte').Snippet;
    [key: string]: any
  }

  let { class: className = undefined, children, ...rest }: Props = $props();

</script>

<div class={cn('flex flex-col space-y-1.5 p-3 sm:p-6 pb-0 sm:pb-0', className)} {...rest}>
  {@render children?.()}
</div>
