<script lang="ts">
import { Separator as SeparatorPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = SeparatorPrimitive.Props

  interface Props {
    class?: $$Props["class"];
    orientation?: $$Props["orientation"];
    decorative?: $$Props["decorative"];
    [key: string]: any
  }

  let { class: className = undefined, orientation = "horizontal", decorative = undefined, ...rest }: Props = $props();

</script>

<SeparatorPrimitive.Root
  class={cn(
    'shrink-0 bg-border',
    orientation === 'horizontal' ? 'h-[1px] w-full' : 'h-full w-[1px]',
    className,
  )}
  {orientation}
  {decorative}
  {...rest}
/>
