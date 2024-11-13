<script lang="ts">
import { Dialog as SheetPrimitive } from "bits-ui"
import X from "virtual:icons/lucide/x"
import { fly } from "svelte/transition"
import { SheetOverlay, SheetPortal, type Side, sheetTransitions, sheetVariants } from "./index.js"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = SheetPrimitive.ContentProps & {
  side?: Side
}


  interface Props {
    class?: $$Props["class"];
    side?: $$Props["side"];
    inTransition?: $$Props["inTransition"];
    inTransitionConfig?: $$Props["inTransitionConfig"];
    outTransition?: $$Props["outTransition"];
    outTransitionConfig?: $$Props["outTransitionConfig"];
    children?: import('svelte').Snippet;
    [key: string]: any
  }

  let {
    class: className = undefined,
    side = "right",
    inTransition = fly,
    inTransitionConfig = sheetTransitions[side ?? "right"].in,
    outTransition = fly,
    outTransitionConfig = sheetTransitions[side ?? "right"].out,
    children,
    ...rest
  }: Props = $props();
</script>

<SheetPortal>
  <SheetOverlay />
  <SheetPrimitive.Content
    {inTransition}
    {inTransitionConfig}
    {outTransition}
    {outTransitionConfig}
    class={cn(sheetVariants({ side }), className)}
    {...rest}
  >
    {@render children?.()}
    <SheetPrimitive.Close
      class="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none data-[state=open]:bg-secondary"
    >
      <X class="h-4 w-4" />
      <span class="sr-only">Close</span>
    </SheetPrimitive.Close>
  </SheetPrimitive.Content>
</SheetPortal>
