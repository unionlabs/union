<script lang="ts">
import { Drawer as DrawerPrimitive } from "vaul-svelte"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = DrawerPrimitive.OverlayProps

  interface Props {
    el?: $$Props["el"];
    class?: $$Props["class"];
    children?: import('svelte').Snippet;
    [key: string]: any
  }

  let { el = $bindable(undefined), class: className = undefined, children, ...rest }: Props = $props();

</script>

<DrawerPrimitive.Overlay
  bind:el
  class={cn('fixed inset-0 z-50 bg-black/80', className)}
  {...rest}
>
  {@render children?.()}
</DrawerPrimitive.Overlay>
