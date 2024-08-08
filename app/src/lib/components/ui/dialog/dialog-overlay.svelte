<script lang="ts">
import { Dialog as DialogPrimitive } from "bits-ui"
import { fade } from "svelte/transition"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = DialogPrimitive.OverlayProps

let className: $$Props["class"] = undefined
export let transition: $$Props["transition"] = fade
export let transitionConfig: $$Props["transitionConfig"] = {
  duration: 150
}
export let backdropFilter: $$Props["style"] = undefined
export { className as class }
</script>

<DialogPrimitive.Overlay
  {transition}
  {transitionConfig}
  style={`backdrop-filter: ${backdropFilter}`}
  class={cn('fixed inset-0 z-40 bg-muted/60', className)}
  {...$$restProps}
/>
