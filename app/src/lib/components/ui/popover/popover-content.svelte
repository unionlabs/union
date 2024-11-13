<script lang="ts">
import { Popover as PopoverPrimitive } from "bits-ui"
import { cn, flyAndScale } from "$lib/utilities/shadcn.js"

type $$Props = PopoverPrimitive.ContentProps
  interface Props {
    class?: $$Props["class"];
    transition?: $$Props["transition"];
    transitionConfig?: $$Props["transitionConfig"];
    children?: import('svelte').Snippet;
    [key: string]: any
  }

  let {
    class: className = undefined,
    transition = flyAndScale,
    transitionConfig = undefined,
    children,
    ...rest
  }: Props = $props();

</script>

<PopoverPrimitive.Content
  {transition}
  {transitionConfig}
  class={cn(
    'z-50 w-72 rounded-md border bg-popover p-4 text-popover-foreground shadow-md outline-none',
    className,
  )}
  {...rest}
>
  {@render children?.()}
</PopoverPrimitive.Content>
