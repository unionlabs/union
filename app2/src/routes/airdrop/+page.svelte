<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import StepperCard from "./StepperCard.svelte"

let currentSlide = $state(0)
let stepperCardRef: StepperCard
</script>

<Sections class="flex items-center w-full h-full">
  <StepperCard
    bind:this={stepperCardRef}
    bind:currentSlide
    totalSlides={4}
  >
    {#snippet children(slideIndex)}
      <div class="flex flex-col p-4 gap-4 h-full">
        {#if slideIndex === 0}
          <h1 class="text-4xl font-bold">Union Airdrop</h1>
          <p class="text-gray-400">This is the first page</p>
          <Button
            class="mt-auto"
            onclick={() => stepperCardRef.goToNextSlide()}
          >Authenticate</Button>
        {:else if slideIndex === 1}
          <h1 class="text-4xl font-bold">Page 2</h1>
          <p class="text-gray-600">This is the second page</p>
          <Button onclick={() => stepperCardRef.goToNextSlide()}>Next</Button>
        {:else if slideIndex === 2}
          <h1 class="text-4xl font-bold">Page 3</h1>
          <p class="text-gray-600">This is the third page</p>
          <Button onclick={() => stepperCardRef.goToNextSlide()}>Next</Button>
        {:else if slideIndex === 3}
          <h1 class="text-4xl font-bold">Page 4</h1>
          <p class="text-gray-600">This is the fourth page</p>
          <div class="flex gap-2">
            <Button onclick={() => stepperCardRef.goToSlide(0)}>Start over</Button>
          </div>
        {/if}
      </div>
    {/snippet}
  </StepperCard>
</Sections>
