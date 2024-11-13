<script lang="ts">
import { Drawer as DrawerPrimitive } from "vaul-svelte"
import DrawerOverlay from "./drawer-overlay.svelte"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = DrawerPrimitive.ContentProps

  interface Props {
    class?: $$Props["class"];
    children?: import('svelte').Snippet;
    [key: string]: any
  }

  let { class: className = undefined, children, ...rest }: Props = $props();

</script>

<DrawerPrimitive.Portal>
  <DrawerOverlay />
  <DrawerPrimitive.Content
    class={cn(
      'fixed inset-x-0 bottom-0 z-50 mt-24 flex h-auto flex-col rounded-t-[10px] border bg-background',
      className,
    )}
    {...rest}
  >
    <div class="mx-auto mt-4 h-2 w-[100px] rounded-full bg-muted"></div>
    {@render children?.()}
  </DrawerPrimitive.Content>
</DrawerPrimitive.Portal>
