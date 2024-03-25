<script lang="ts">
import { Dialog as SheetPrimitive } from "bits-ui"
import X from "lucide-svelte/icons/x"
import { fly } from "svelte/transition"
import { SheetOverlay, SheetPortal, type Side, sheetTransitions, sheetVariants } from "./index.js"
import { cn } from "$/lib/utilities/shadcn.js"

type $$Props = SheetPrimitive.ContentProps & {
  side?: Side
}

let className: $$Props["class"] = undefined
export let side: $$Props["side"] = "right"
export { className as class }
export let inTransition: $$Props["inTransition"] = fly
export let inTransitionConfig: $$Props["inTransitionConfig"] = sheetTransitions[side ?? "right"].in
export let outTransition: $$Props["outTransition"] = fly
export let outTransitionConfig: $$Props["outTransitionConfig"] =
  sheetTransitions[side ?? "right"].out
</script>

<SheetPortal>
  <SheetOverlay />
  <SheetPrimitive.Content
    {inTransition}
    {inTransitionConfig}
    {outTransition}
    {outTransitionConfig}
    class={cn(sheetVariants({ side }), className)}
    {...$$restProps}
  >
    <slot />
    <SheetPrimitive.Close
      class="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none data-[state=open]:bg-secondary"
    >
      <X class="h-4 w-4" />
      <span class="sr-only">Close</span>
    </SheetPrimitive.Close>
  </SheetPrimitive.Content>
</SheetPortal>
