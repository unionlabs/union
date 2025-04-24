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

<div class="flex flex-1 gap-4 items-center px-4 py-0.5 border-b border-zinc-800">
  <div
    class={cn(
      "flex items-center w-full transition-all duration-500 py-4",
      className
    )}
    {...rest}
  >
    <div class="step-balls-container flex items-center w-full">
      {#each Array(totalSteps) as _, i (i)}
        <div
          class={cn(
            "flex items-center transition-all duration-500",
            i < totalSteps - 1 ? "flex-1" : ""
          )}
        >
          <!-- Step dot with tooltip -->
          <Tooltip>
            {#snippet trigger()}
              <div
                class={cn(
                  "step-ball w-3 h-3 rounded-full flex items-center justify-center transition-all duration-300 cursor-help",
                  i + 1 < currentStep
                    ? "bg-white completed"
                    : i + 1 === currentStep
                      ? "bg-white current ring-1 ring-white/30"
                      : "bg-zinc-600"
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
</div>

<style>
  .step-ball {
    background-color: rgb(82 82 91); /* zinc-600 */
    animation: initial-render 0.35s ease-out;
  }

  .step-ball.completed {
    background-color: white;
  }

  .step-ball.current {
    background-color: white;
    box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.3);
  }

  /* Only apply blink animation to non-completed, non-current balls */
  .step-ball:not(.completed):not(.current) {
    animation:
      initial-render 0.35s ease-out,
      blink 0.3s ease-in-out 0.35s;
    animation-fill-mode: forwards;
  }

  @keyframes initial-render {
    0% {
      opacity: 0;
      transform: scale(0.8);
    }
    100% {
      opacity: 1;
      transform: scale(1);
    }
  }

  @keyframes blink {
    0% {
      background-color: rgb(82 82 91);
    }
    50% {
      background-color: white;
      transform: scale(1.1);
    }
    100% {
      background-color: rgb(82 82 91);
      transform: scale(1);
    }
  }

  /* Delay each ball's blink animation */
  .step-balls-container
    > div:nth-child(1)
    .step-ball:not(.completed):not(.current) {
    animation-delay: 0s, 0.1s;
  }
  .step-balls-container
    > div:nth-child(2)
    .step-ball:not(.completed):not(.current) {
    animation-delay: 0s, 0.2s;
  }
  .step-balls-container
    > div:nth-child(3)
    .step-ball:not(.completed):not(.current) {
    animation-delay: 0s, 0.3s;
  }
  .step-balls-container
    > div:nth-child(4)
    .step-ball:not(.completed):not(.current) {
    animation-delay: 0s, 0.4s;
  }
  .step-balls-container
    > div:nth-child(5)
    .step-ball:not(.completed):not(.current) {
    animation-delay: 0s, 0.5s;
  }
  .step-balls-container
    > div:nth-child(6)
    .step-ball:not(.completed):not(.current) {
    animation-delay: 0s, 0.6s;
  }
  .step-balls-container
    > div:nth-child(7)
    .step-ball:not(.completed):not(.current) {
    animation-delay: 0s, 0.7s;
  }
  .step-balls-container
    > div:nth-child(8)
    .step-ball:not(.completed):not(.current) {
    animation-delay: 0s, 0.8s;
  }
  .step-balls-container
    > div:nth-child(9)
    .step-ball:not(.completed):not(.current) {
    animation-delay: 0s, 0.9s;
  }
  .step-balls-container
    > div:nth-child(10)
    .step-ball:not(.completed):not(.current) {
    animation-delay: 0s, 1s;
  }
</style>
