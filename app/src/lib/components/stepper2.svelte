<script lang="ts">
import { cn } from "$lib/utilities/shadcn"
import Button from "$lib/components/ui/button/button.svelte"
import { type Readable, derived } from "svelte/store"
import SpinnerSvg from "./spinner-svg.svelte"
import type { Step, RawTrace } from "$lib/stepper-types.ts"
import { toIsoString } from "$lib/utilities/date"
import Truncate from "$lib/components/truncate.svelte"
import { createEventDispatcher } from "svelte"

export let steps: Readable<Array<RawTrace>>

// const dispatch = createEventDispatcher()
//
let pSteps = derived(steps, $steps => {
  let processedSteps = $steps
  // patch gaps (see #2544)
  for (const [index, step] of processedSteps.entries()) {
    const gap = processedSteps.slice(index).find(step => step.status === "COMPLETED") !== undefined
    if (gap && (step.status === "IN_PROGRESS" || step.status === "PENDING")) {
      processedSteps[index].status = "COMPLETED"
    }
  }
  return processedSteps
})
//
// const cancel = () => {
//   dispatch("cancel")
// }
</script>

<ol class="max-w-full w-full -my-4"> <!-- offset padding surplus !-->
{#each $steps as step, index}
  <li class="flex gap-4 w-full">
    <div class="flex flex-col items-center">
      <!-- top step connector !-->
      <div class={cn(
          "w-1 flex-1",
          index !== 0 ?  "dark:bg-muted-foreground bg-black" : "",
          index !== 0 ?  "dark:bg-muted-foreground bg-black" : "",
          )}></div>
      <!-- stepper icon !-->
      <div class={cn(
        "size-12 border-4 relative transition-all duration-300",
        step.status === "PENDING" ? "bg-white" :
        step.status === "IN_PROGRESS" ? "bg-white" :
        step.status === "COMPLETED" ? "bg-accent" :
        step.status === "ERROR" ? "bg-black" :
        step.status === "WARNING" ? "bg-yellow-300" : ""
      )}>
        <div class={cn("absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2  rounded-full bg-black transition-all duration-300",
          step.status === "COMPLETED" ? "w-1 h-7 rotate-45 translate-x-[2px]" :
          step.status === "ERROR" ? "w-1 h-8 rotate-45 bg-white" :
          step.status === "WARNING" ? "w-1 h-4 -translate-y-[12px]" : "w-2 h-2"
          )}></div>
        <div class={cn("absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 rounded-full bg-black transition-all duration-300",
          step.status === "COMPLETED" ? "w-1 h-4 -rotate-45 -translate-x-3 -translate-y-[2px]" :
          step.status === "ERROR" ? "w-1 h-8 -rotate-45 bg-white" :
          step.status === "WARNING" ? "w-1 h-1 translate-y-[8px]" : "w-2 h-2"
          )}></div>
        {#if step.status === "IN_PROGRESS"}
          <SpinnerSvg className="absolute text-accent w-8 h-8 left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2"/>
        {/if}
      </div>
      <!-- bottom step connector !-->
      <div class={cn("w-1 flex-1",
      index === $stepsUpToError.length - 1 ? "bg-transparent" : "dark:bg-muted-foreground",
      index !== $steps.length - 1  &&
      step.status !== "ERROR" &&
      step.status !== "WARNING" ?  "bg-black" : "")
      }></div>
    </div>
    <div class="font-bold py-4 flex flex-col min-h-[80px] max-w-[calc(100%-80px)] break-words justify-center">
      {#if step.traceDetails}
        {@const trace = step.traceDetails}
        <p class="text-xs -mb-1 text-muted-foreground">{toIsoString(new Date(trace.timestamp)).split('T')[1]} on {trace.chain_display_name} at {#if trace.block_url}<a class="underline" href={trace.block_url}>{trace.block}</a>{:else}{trace.block}{/if}</p>
      {/if}
      <div>{step.title}</div>
      {#if step.traceDetails}
        {@const trace = step.traceDetails}
        {#if trace.tx_url !== undefined}
          <a href={trace.tx_url} class="-mt-1 block underline text-xs text-muted-foreground"><Truncate class="underline" value={trace.tx} type="hash"/></a>
        {:else}
          <p class="text-xs text-muted-foreground"><Truncate value={trace.tx} type="hash"/></p>
        {/if}
      {:else if step.description}
          <div class="font-normal break-words">{step.description}</div>
      {/if}
    </div>
  </li>
{/each}
</ol>


{#if $stepsUpToError.length < $steps.length && onRetry !== undefined}

  <div class="flex gap-1 mt-6 w-full">

    <Button
      variant="default"
      on:click={onRetry}
      class='!hover:bg-foreground !hover:text-primary-foreground hover:text-accent w-full'
    >
      {$stepsUpToError.slice(-1)[0].status === "WARNING" ? "CONTINUE" : "RETRY" }
      
    </Button>

    <Button
      on:click={cancel}
      variant="outline"
      class='text-primary text-md font-bold !hover:bg-foreground !hover:text-primary-foreground w-full'
    >
      CANCEL
    </Button>
  </div>

{/if}
