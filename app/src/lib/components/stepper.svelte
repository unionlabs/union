<script lang="ts">
import { cn } from "$lib/utilities/shadcn"
import Button from "$lib/components/ui/button/button.svelte"
import { type Readable, derived } from "svelte/store"

type Step = {
  status: StepStatus
  title: string
  description: string
}

type StepStatus = "PENDING" | "IN_PROGRESS" | "COMPLETED" | "ERROR"

export let steps: Readable<Array<Step>>

let stepsUpToError = derived(steps, $steps => {
  let errorIndex = $steps.findIndex(step => step.status === "ERROR")
  return errorIndex === -1 ? $steps : $steps.slice(0, errorIndex + 1)
})

export let onRetry: (() => void) | undefined
</script>

<ol>
{#each $stepsUpToError as step, index}
  <li class="flex gap-4">
    <div class="flex flex-col items-center">
      <!-- top step connector !-->
      <div class={cn("w-1 flex-1", index !== 0 ?  "bg-black" : "")}></div>
      <!-- stepper icon !-->
      <div class={cn(
        "size-12 border-4 flex items-center justify-center transition-colors",
        step.status === "PENDING" ? "bg-white" : 
        step.status === "IN_PROGRESS" ? "bg-muted" :
        step.status === "COMPLETED" ? "bg-accent" :
        step.status === "ERROR" ? "bg-black" : ""
      )}>
        <div class="rounded-full bg-black size-3"></div>
      </div>
      <!-- bottom step connector !-->
      <div class={cn("w-1 flex-1", index !== $steps.length - 1  && step.status !== "ERROR" ?  "bg-black" : "")}></div>
    </div>
    <div class="font-bold py-4 flex flex-col min-h-[80px] justify-center">
      <div>{step.title}</div>
      {#if step.description}<div class="font-normal">{step.description}</div>{/if}
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


