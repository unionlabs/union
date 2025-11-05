<script lang="ts">
import Sections from "$lib/components/ui/Sections.svelte"
import StepperCard from "$lib/components/ui/StepperCard.svelte"
import { Option } from "effect"
import Step1 from "./step/Step1.svelte"
import Step2 from "./step/Step2.svelte"
import Step3 from "./step/Step3.svelte"
let currentSlide = $state(0)
let walletAddress = $state("")
let btcfiPoints = $state<Option.Option<number>>(Option.none())
let stepperCardRef: StepperCard

function goToNextSlide() {
  stepperCardRef.goToNextSlide()
}

const handleStep1Next = (address: string, points: number) => {
  walletAddress = address
  btcfiPoints = Option.some(points)
  goToNextSlide()
}

const handleStep2Back = () => {
  currentSlide = 0
  walletAddress = ""
  btcfiPoints = Option.none()
}

const handleStep2ReceiveEthereum = () => {
  goToNextSlide()
}

const handleStep3Back = () => {
  currentSlide = 1
}

const handleStep3Success = () => {
  // Reset to beginning after successful verification
  currentSlide = 0
  walletAddress = ""
  btcfiPoints = Option.none()
}
</script>

<Sections>
  <div class="flex justify-center items-center h-full w-full">
    <StepperCard
      bind:this={stepperCardRef}
      bind:currentSlide
      totalSlides={3}
      class="max-w-5xl md:h-auto"
    >
      {#snippet children(slideIndex)}
        <div class="flex flex-col gap-4 h-full w-full">
          {#if slideIndex === 0}
            <Step1 onNext={handleStep1Next} />
          {:else if slideIndex === 1 && Option.isSome(btcfiPoints)}
            <Step2
              {walletAddress}
              {btcfiPoints}
              onBack={handleStep2Back}
              onReceiveEthereum={handleStep2ReceiveEthereum}
            />
          {:else if slideIndex === 2 && Option.isSome(btcfiPoints)}
            <Step3
              {walletAddress}
              {btcfiPoints}
              onBack={handleStep3Back}
              onSuccess={handleStep3Success}
            />
          {/if}
        </div>
      {/snippet}
    </StepperCard>
  </div>
</Sections>
