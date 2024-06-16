<script lang="ts">
import { Dialog as DialogPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = DialogPrimitive.TitleProps

let className: $$Props["class"] = undefined
export { className as class }
</script>

<DialogPrimitive.Title
  class={cn('text-xl uppercase font-bold font-supermolot', className)}
  {...$$restProps}
>
  <slot />
</DialogPrimitive.Title>
