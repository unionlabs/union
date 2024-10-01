<script lang="ts">
import Print from "$lib/components/Terminal/Print.svelte"
import { getState } from "$lib/state/index.svelte.ts"

const { terminal, contributor } = getState()

let progressBar = $derived(
  "#".repeat(terminal.currentStep) + "=".repeat(terminal.maxStep - terminal.currentStep)
)
</script>

<div class="py-2 px-4 border-t border-[#48494C] w-full flex justify-between">
  <div class="flex items-center gap-2">
    <div class="text-center font-mono">
      <Print>[<span class="text-union-accent-500">{progressBar}</span>]</Print>
    </div>
    <Print>{terminal.currentStep}/{terminal.maxStep} COMPLETED</Print>
  </div>

  {#if contributor.clientState !== "offline" && contributor.clientState !== undefined}
    <div class="flex items-center gap-2">
      <div class="w-2 h-2 bg-union-accent-500 rounded-full"></div>
      <Print>CONNECTED</Print>
    </div>
    {:else }
    <div class="flex items-center gap-2">
      <div class="w-2 h-2 bg-[#FD6363] rounded-full"></div>
      <Print>DISCONNECTED</Print>
    </div>
  {/if}
</div>