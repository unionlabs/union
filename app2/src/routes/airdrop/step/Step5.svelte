<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import StepLayout from "../StepLayout.svelte"

interface Props {
  onNext?: () => void
  onRestart?: () => void
}

let { onNext, onRestart }: Props = $props()
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-4 z-10 justify-between h-full">
      <div class="space-y-4">
        <div>
          <h1 class="text-4xl font-semibold">
            Final Step
          </h1>
          <p class="text-gray-400 mt-2">
            Complete your airdrop journey
          </p>
        </div>
      </div>
      
      <div class="space-y-3">
        {#if onRestart}
          <Button
            size="md"
            variant="secondary"
            class="flex w-full items-center justify-center gap-3"
            onclick={onRestart}
          >
            Start Over
          </Button>
        {/if}
        {#if onNext}
          <Button
            size="md"
            variant="primary"
            class="flex w-full items-center justify-center gap-3"
            onclick={onNext}
          >
            Finish
          </Button>
        {/if}
      </div>
    </div>
  {/snippet}
  
  {#snippet right()}
    <div class="aspect-square relative flex items-center justify-center">
      <div class="w-full h-full flex items-center justify-center bg-gradient-to-br from-orange-500/10 to-red-500/10 rounded-lg border border-zinc-700">
        <h2 class="text-2xl font-semibold text-center">
          Step 5 Content
        </h2>
      </div>
    </div>
  {/snippet}
</StepLayout> 