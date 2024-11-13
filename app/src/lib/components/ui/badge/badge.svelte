<script lang="ts">
import { type Variant, badgeVariants } from "./index.js"
import { cn } from "$lib/utilities/shadcn.js"

  interface Props {
    class?: string | undefined | null;
    href?: string | undefined;
    variant?: Variant;
    children?: import('svelte').Snippet;
    [key: string]: any
  }

  let {
    class: className = undefined,
    href = undefined,
    variant = "default",
    children,
    ...rest
  }: Props = $props();

</script>

<svelte:element
  this={href ? 'a' : 'span'}
  {href}
  class={cn(badgeVariants({ variant, className }))}
  {...rest}
>
  {@render children?.()}
</svelte:element>
