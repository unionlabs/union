<script lang="ts">
import { Dialog as DialogPrimitive } from "bits-ui"
import X from "virtual:icons/lucide/x"
import * as Dialog from "./index.js"
import { cn, flyAndScale } from "$lib/utilities/shadcn.js"

type $$Props = DialogPrimitive.ContentProps



  interface Props {
    class?: $$Props["class"];
    overlayClass?: $$Props["class"];
    transition?: $$Props["transition"];
    transitionConfig?: $$Props["transitionConfig"];
    backdropFilter?: $$Props["style"];
    children?: import('svelte').Snippet;
    [key: string]: any
  }

  let {
    class: className = undefined,
    overlayClass = undefined,
    transition = flyAndScale,
    transitionConfig = {
  duration: 200
},
    backdropFilter = undefined,
    children,
    ...rest
  }: Props = $props();


</script>

<Dialog.Portal>
  <Dialog.Overlay
    data-dialog-overlay
    class={cn(overlayClass)}
    style={`backdrop-filter: ${backdropFilter}`}
  />
  <DialogPrimitive.Content
    {transition}
    {transitionConfig}
    data-dialog-content
    class={cn(
      "border bg-background shadow-lg",
      "fixed left-[50%] top-[50%] z-40 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 p-4 sm: md:w-full",
      className,
    )}
    {...rest}
  >
    {@render children?.()}
    <DialogPrimitive.Close
      class={cn(
        "absolute right-3 top-4 rounded-sm focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none",
        "opacity-70 ring-offset-background transition-opacity hover:opacity-100 data-[state=open]:bg-accent data-[state=open]:text-muted-foreground"
      )}
    >
      <X class="text-white size-4 -mt-0.5" />
      <span class="sr-only">Close</span>
    </DialogPrimitive.Close>
  </DialogPrimitive.Content>
</Dialog.Portal>
