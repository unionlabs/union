<script lang="ts">
import { Select as SelectPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = SelectPrimitive.LabelProps

  interface Props {
    class?: $$Props["class"];
    children?: import('svelte').Snippet;
    [key: string]: any
  }

  let { class: className = undefined, children, ...rest }: Props = $props();

</script>

<SelectPrimitive.Label
  class={cn('py-1.5 pl-8 pr-2 text-sm font-semibold', className)}
  {...rest}
>
  {@render children?.()}
</SelectPrimitive.Label>
