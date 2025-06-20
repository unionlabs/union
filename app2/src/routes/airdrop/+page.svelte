<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte";
import Sections from "$lib/components/ui/Sections.svelte"
import StepperCard from "./StepperCard.svelte"
import Step1 from "./step/Step1.svelte"

let currentSlide = $state(0)
let stepperCardRef: StepperCard
</script>

<Sections class="flex items-center justify-center w-full h-full -mt-12">
  <StepperCard
    bind:this={stepperCardRef}
    bind:currentSlide
    totalSlides={5}
    class="max-w-3xl h-full md:h-auto"
  >
    {#snippet children(slideIndex)}
            <div class="flex flex-col gap-4 h-full w-full">
        {#if slideIndex === 0}
          <Step1 onNext={() => stepperCardRef.goToNextSlide()} />
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


