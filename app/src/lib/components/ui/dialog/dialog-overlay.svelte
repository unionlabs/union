<script lang="ts">
import { Dialog as DialogPrimitive } from "bits-ui"
import { fade } from "svelte/transition"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = DialogPrimitive.OverlayProps

  interface Props {
    class?: $$Props["class"];
    transition?: $$Props["transition"];
    transitionConfig?: $$Props["transitionConfig"];
    backdropFilter?: $$Props["style"];
    [key: string]: any
  }

  let {
    class: className = undefined,
    transition = fade,
    transitionConfig = {
  duration: 150
},
    backdropFilter = undefined,
    ...rest
  }: Props = $props();

</script>

<DialogPrimitive.Overlay
  {transition}
  {transitionConfig}
  style={`backdrop-filter: ${backdropFilter}`}
  class={cn('fixed inset-0 z-40 bg-muted/60', className)}
  {...rest}
/>
