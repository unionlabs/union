<script lang="ts">

import Sections from "$lib/components/ui/Sections.svelte"
import StepperCard from "./StepperCard.svelte"
import Step1 from "./step/Step1.svelte"
import Step2 from "./step/Step2.svelte"
import Step3 from "./step/Step3.svelte"
import Step4 from "./step/Step4.svelte"
import Step5 from "./step/Step5.svelte"

let currentSlide = $state(0)
let stepperCardRef: StepperCard
</script>

<Sections class="flex items-center justify-center   w-full h-full sm:h-[calc(100%-64px)] ">
  <StepperCard
    bind:this={stepperCardRef}
    bind:currentSlide
    totalSlides={5}
    class="max-w-3xl  md:h-auto"
  >
    {#snippet children(slideIndex)}
      <div class="flex flex-col gap-4 h-full w-full">
        {#if slideIndex === 0}
          <Step1 onNext={() => stepperCardRef.goToNextSlide()} />
        {:else if slideIndex === 1}
          <Step2 onNext={() => stepperCardRef.goToNextSlide()} />
        {:else if slideIndex === 2}
          <Step3 onNext={() => stepperCardRef.goToNextSlide()} />
        {:else if slideIndex === 3}
          <Step4 onNext={() => stepperCardRef.goToNextSlide()} />
        {:else if slideIndex === 4}
          <Step5 onRestart={() => stepperCardRef.goToSlide(0)} />
        {/if}
      </div>
    {/snippet}
  </StepperCard>
</Sections>


