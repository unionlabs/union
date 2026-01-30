<script lang="ts">
import { cn } from "$lib/utils.js"
import { Avatar as AvatarPrimitive } from "bits-ui"

let {
  ref = $bindable(null),
  class: className,
  ...restProps
}: AvatarPrimitive.FallbackProps = $props()
</script>

<AvatarPrimitive.Fallback
  bind:ref
  data-slot="avatar-fallback"
  class={cn("bg-muted flex size-full items-center justify-center rounded-full", className)}
  {...restProps}
/>
