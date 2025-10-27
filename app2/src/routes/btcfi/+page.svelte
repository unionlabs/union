<script lang="ts">
import Sections from "$lib/components/ui/Sections.svelte"
import StepperCard from "$lib/components/ui/StepperCard.svelte"
import Step1 from "./step/Step1.svelte"
import Step2 from "./step/Step2.svelte"
let currentSlide = $state(0)
let walletAddress = $state("")
let btcfiPoints = $state<number | null>(null)
let stepperCardRef: StepperCard

function goToNextSlide() {
  stepperCardRef.goToNextSlide()
}

const handleStep1Next = (address: string, points: number) => {
  walletAddress = address
  btcfiPoints = points
  goToNextSlide()
}

const handleStep2Back = () => {
  currentSlide = 0
  walletAddress = ""
  btcfiPoints = null
}
</script>

<Sections>
  <div class="flex justify-center items-center h-full w-full">
    <StepperCard
      bind:this={stepperCardRef}
      bind:currentSlide
      totalSlides={2}
      class="max-w-5xl md:h-auto"
    >
      {#snippet children(slideIndex)}
        <div class="flex flex-col gap-4 h-full w-full">
          {#if slideIndex === 0}
            <Step1 onNext={handleStep1Next} />
          {:else if slideIndex === 1 && btcfiPoints !== null}
            <Step2
              {walletAddress}
              {btcfiPoints}
              onBack={handleStep2Back}
            />
          {/if}
        </div>
      {/snippet}
    </StepperCard>
  </div>
</Sections>
