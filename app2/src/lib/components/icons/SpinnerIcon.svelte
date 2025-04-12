<script lang="ts">
import { cn } from "$lib/utils"

type Props = { class: string }
const { class: className }: Props = $props()
</script>

<div
        class={cn(
    "animate-spin rounded-full border-y-2 border-babylon-orange",
    className
  )}
></div>
