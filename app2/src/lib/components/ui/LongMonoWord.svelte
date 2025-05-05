<script lang="ts">
import { cn } from "$lib/utils"
import type { Snippet } from "svelte"
import type { HTMLAttributes } from "svelte/elements"

type Props = HTMLAttributes<HTMLDivElement> & {
  children: Snippet
  class?: string
}

const { children, class: className = "", ...rest }: Props = $props()
</script>

<div
  class={cn("text-xs font-mono break-all", className)}
  {...rest}
>
  {@render children()}
</div>
