<script lang="ts">
import { AlertDialog as AlertDialogPrimitive } from "bits-ui"
import { buttonVariants } from "$lib/components/ui/button/index.js"
import { cn } from "$lib/utilities/shadcn.js"

let {
  ref = $bindable(null),
  class: className,
  ...restProps
}: AlertDialogPrimitive.ActionProps = $props()
</script>

<AlertDialogPrimitive.Action bind:ref class={cn(buttonVariants(), className)} {...restProps} />
