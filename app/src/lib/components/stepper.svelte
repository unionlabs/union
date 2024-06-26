<script lang="ts">
import { cn } from "$lib/utilities/shadcn"
import Button from "$lib/components/ui/button/button.svelte"
import { type Readable, derived } from "svelte/store"
import SpinnerSvg from "./spinner-svg.svelte"
import type { Step, StepStatus } from "$lib/stepper-types.ts"
import { toIsoString } from "$lib/utilities/date"

export let steps: Readable<Array<Step>>

let stepsUpToError = derived(steps, $steps => {
  let errorIndex = $steps.findIndex(step => step.status === "ERROR")
  return errorIndex === -1 ? $steps : $steps.slice(0, errorIndex + 1)
})

export let onRetry: (() => void) | undefined = undefined
</script>

<ol class="max-w-full">
{#each $stepsUpToError as step, index}
  <li class="flex gap-4 max-w-full">
    <div class="flex flex-col items-center">
      <!-- top step connector !-->
      <div class={cn("w-1 flex-1", index !== 0 ?  "bg-black" : "")}></div>
      <!-- stepper icon !-->
      <div class={cn(
        "size-12 border-4 relative transition-all duration-300",
        step.status === "PENDING" ? "bg-white" : 
        step.status === "IN_PROGRESS" ? "bg-white" :
        step.status === "COMPLETED" ? "bg-accent" :
        step.status === "ERROR" ? "bg-black" : ""
      )}>
        <div class={cn("absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2  rounded-full bg-black transition-all duration-300", 
          step.status === "COMPLETED" ? "w-1 h-7 rotate-45 translate-x-[2px]" : 
          step.status === "ERROR" ? "w-1 h-8 rotate-45 bg-white" : "w-2 h-2"
          )}></div>
        <div class={cn("absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 rounded-full bg-black transition-all duration-300", 
          step.status === "COMPLETED" ? "w-1 h-4 -rotate-45 -translate-x-3 -translate-y-[2px]" : 
          step.status === "ERROR" ? "w-1 h-8 -rotate-45 bg-white" : "w-2 h-2"
          )}></div>
        {#if step.status === "IN_PROGRESS"}
          <SpinnerSvg className="absolute text-accent w-8 h-8 left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2"/>
        {/if}
      </div>
      <!-- bottom step connector !-->
      <div class={cn("w-1 flex-1", index !== $steps.length - 1  && step.status !== "ERROR" ?  "bg-black" : "")}></div>
    </div>
    <div class="font-bold py-4 flex flex-col min-h-[80px] max-w-[calc(100%-50px)] break-words justify-center">
      {#if step.traceDetails}
        {@const trace = step.traceDetails}
        <p class="text-xs -mb-1 text-muted-foreground">{toIsoString(new Date(trace.timestamp)).split('T')[1]} on {trace.chain_display_name} at {#if trace.block_url}<a class="underline" href={trace.block_url}>{trace.block}</a>{:else}{trace.block}{/if}</p>
      {/if} 
      <div>{step.title}</div>
      {#if step.traceDetails}
        {@const trace = step.traceDetails}
        {#if trace.tx_url !== undefined}
          <a href={trace.tx_url} class="-mt-1 block underline text-xs text-muted-foreground">{trace.tx}</a>
        {:else}
          <p class="text-xs text-muted-foreground">{trace.tx}</p>
        {/if}
      {:else if step.description}
          <div class="font-normal break-words">{step.description}</div>      
      {/if} 
    </div>
  </li>
{/each}
</ol>
{#if $stepsUpToError.length < $steps.length && onRetry !== undefined}

      <Button
        size="default"
        variant="link"
        on:click={onRetry}
        class='bg-foreground text-primary-foreground !hover:bg-foreground !hover:text-primary-foreground'
      >
        Retry
      </Button>


{/if}


