<script lang="ts">
import { cn } from "$lib/utils"
import type { HTMLAttributes } from "svelte/elements"

type Props = HTMLAttributes<HTMLDivElement> & {
  class?: string
  title?: string
  description?: string
}

const {
  class: className = "",
  title = "No Wallet Connected",
  description = "Connect a wallet to view your transfers.",
  ...rest
}: Props = $props()

const containerClasses = cn("h-[600px] flex justify-center items-center", className)
</script>

<div class={containerClasses} {...rest}>
  <div class="flex flex-col items-center">
    <h2 class="text-xl font-bold">{title}</h2>
    <p>{description}</p>
  </div>
</div>
