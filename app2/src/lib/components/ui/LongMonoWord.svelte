<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import type { Snippet } from "svelte"
import { cn } from "$lib/utils"

type Props = HTMLAttributes<HTMLDivElement> & {
  children: Snippet
  class?: string
}

const { children, class: className = "", ...rest }: Props = $props()
</script>

<div class={cn("text-sm font-mono break-all", className)} {...rest}>
  {@render children()}
</div>
