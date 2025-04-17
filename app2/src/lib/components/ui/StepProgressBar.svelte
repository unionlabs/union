<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import { cn } from "$lib/utils"
import Tooltip from "$lib/components/ui/Tooltip.svelte"

type Props = HTMLAttributes<HTMLDivElement> & {
  currentStep: number
  totalSteps: number
  class?: string
  stepDescriptions?: Array<string>
}

const {
  currentStep = 1,
  totalSteps = 1,
  stepDescriptions = [],
  class: className = "",
  ...rest
}: Props = $props()
</script>

<div class="flex flex-1 gap-4 items-center px-4 border-b border-zinc-800">
  <div class={cn("flex items-center w-full transition-all duration-500 py-4", className)} {...rest}>
    {#each Array(totalSteps) as _, i (i)}
      <div class={cn("flex items-center transition-all duration-500", i < totalSteps - 1 ? "flex-1" : "")}>
        <!-- Step dot with tooltip -->
        <Tooltip>
          {#snippet trigger()}
          <div
            class={cn(
              "w-3 h-3 rounded-full flex items-center justify-center transition-all duration-300 cursor-help",
              i + 1 < currentStep ? "bg-white" :
              i + 1 === currentStep ? "bg-white ring-1 ring-white/30" :
              "bg-zinc-600"
            )}
          ></div>
          {/snippet}

          {#snippet content()}
          <div class="text-sm p-1">
            <div class="font-medium">Step {i + 1}</div>
            <div class="text-zinc-300">
              {stepDescriptions[i] || `Step ${i + 1}`}
            </div>
          </div>
          {/snippet}
        </Tooltip>

        <!-- Connector line (except for last item) -->
        {#if i < totalSteps - 1}
          <div
            class={cn(
            "h-[1px] flex-1 mx-1 transition-all duration-300",
            i + 1 < currentStep ? "bg-white" : "bg-zinc-600"
          )}
          ></div>
        {/if}
      </div>
    {/each}
  </div>
</div>
