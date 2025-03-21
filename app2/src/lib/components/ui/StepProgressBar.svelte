<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import { cn } from "$lib/utils"

type Props = HTMLAttributes<HTMLDivElement> & {
  currentStep: number
  totalSteps: number
  class?: string
}

const { currentStep = 1, totalSteps = 1, class: className = "", ...rest }: Props = $props()
</script>

<div class={cn("flex items-center w-full transition-all duration-500", className)} {...rest}>
  {#each Array(totalSteps) as _, i (i)}
    <div class={cn("flex items-center transition-all duration-500", i < totalSteps - 1 ? "flex-1" : "")}>
      <!-- Step dot -->
      <div 
        class={cn(
          "w-4 h-4 rounded-full flex items-center justify-center transition-all duration-300",
          i + 1 < currentStep ? "bg-white" : 
          i + 1 === currentStep ? "bg-white ring-1 ring-white/30" : 
          "bg-zinc-600"
        )}
      ></div>
      
      <!-- Connector line (except for last item) -->
      {#if i < totalSteps - 1}
        <div 
          class={cn(
            "h-[2px] flex-1 mx-1 transition-all duration-300",
            i + 1 < currentStep ? "bg-white" : "bg-zinc-600"
          )}
        ></div>
      {/if}
    </div>
  {/each}
</div>
