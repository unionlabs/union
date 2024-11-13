<script lang="ts">
import { Drawer as DrawerPrimitive } from "vaul-svelte"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = DrawerPrimitive.DescriptionProps

  interface Props {
    el?: $$Props["el"];
    class?: $$Props["class"];
    children?: import('svelte').Snippet;
    [key: string]: any
  }

  let { el = $bindable(undefined), class: className = undefined, children, ...rest }: Props = $props();

</script>

<DrawerPrimitive.Description
  bind:el
  class={cn('text-sm text-muted-foreground', className)}
  {...rest}
>
  {@render children?.()}
</DrawerPrimitive.Description>
