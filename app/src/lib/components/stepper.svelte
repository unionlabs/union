<script lang="ts">
    import { cn } from "$lib/utilities/shadcn";



type Step = {
  status: StepStatus
}

type StepStatus = "PENDING" | "IN_PROGRESS" | "COMPLETED" | "ERROR";

let steps: Array<Step> = [
  { 
    status: "PENDING",
  },
  { 
    status: "IN_PROGRESS",
  },
  { 
    status: "COMPLETED",
  },
  { 
    status: "ERROR",
  }
]
</script>


<ol>
{#each steps as step, index}
  <li class="flex gap-4">
    <div class="flex flex-col items-center">
      <!-- top step connector !-->
      <div class={cn("w-1 flex-1", index !== 0 ?  "bg-black" : "")}></div>
      <!-- stepper icon !-->
      <div class={cn(
        "w-12 h-12 border-4 flex items-center justify-center",
        step.status === "PENDING" ? "bg-white" : 
        step.status === "IN_PROGRESS" ? "bg-muted" :
        step.status === "COMPLETED" ? "bg-accent" :
        step.status === "ERROR" ? "bg-black" : ""
      )}>
        <div class="rounded-full bg-black w-3 h-3"></div>
      </div>
      <!-- bottom step connector !-->
      <div class={cn("w-1 flex-1", index !== steps.length - 1 ?  "bg-black" : "")}></div>
    </div>
    <div class="font-bold py-4">
    {step.status}
    </div>


  </li>
{/each}
</ol>
