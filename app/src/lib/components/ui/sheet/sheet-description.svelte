<script lang="ts">
import { Dialog as SheetPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = SheetPrimitive.DescriptionProps

  interface Props {
    class?: $$Props["class"];
    children?: import('svelte').Snippet;
    [key: string]: any
  }

  let { class: className = undefined, children, ...rest }: Props = $props();

</script>

<SheetPrimitive.Description class={cn('text-sm text-muted-foreground', className)} {...rest}>
  {@render children?.()}
</SheetPrimitive.Description>
