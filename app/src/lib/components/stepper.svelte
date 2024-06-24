<script lang="ts">
import { cn } from "$lib/utilities/shadcn"

type Step = {
  status: StepStatus
  title: string
  description: string
}

type StepStatus = "PENDING" | "IN_PROGRESS" | "COMPLETED" | "ERROR"

export let steps: Array<Step>
</script>


<ol>
{#each steps as step, index}
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
      <div class={cn("w-1 flex-1", index !== steps.length - 1 ?  "bg-black" : "")}></div>
    </div>
    <div class="font-bold py-4 flex flex-col min-h-[80px] justify-center">
      <div>{step.title}</div>
      {#if step.description}<div class="font-normal">{step.description}</div>{/if}
    </div>
  </li>
{/each}
</ol>
