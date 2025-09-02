<script lang="ts">
import { goto } from "$app/navigation"
import { onMount } from "svelte"
// import { browser } from "$app/environment"
// import { page } from "$app/state"
// import StepperCard from "$lib/components/ui/StepperCard.svelte"
// import { dashboard } from "$lib/dashboard/stores/user.svelte"
// import { Option } from "effect"
// import Step1 from "./step/Step1.svelte"
// import Step2 from "./step/Step2.svelte" // Terms of Service
// import Step3 from "./step/Step3.svelte" // EVM wallet connection
// import Step4 from "./step/Step4.svelte" // Twitter connection
// import Step5 from "./step/Step5.svelte" // Cosmos connection
// import Step6 from "./step/Step6.svelte" // Profile scanning
// import Step7 from "./step/Step7.svelte" // Allocation display
// import Step8 from "./step/Step8.svelte" // Human verification (final step)

onMount(() => {
  goto("/udrop", { replaceState: true })
})

// let currentSlide = $state(0)
// let stepperCardRef: StepperCard
// let isInitialized = $state(false)

// // Initialize from URL search params (only once) with auth guard
// $effect(() => {
//   if (browser && !isInitialized) {
//     const step = page.url.searchParams.get("step")
//     if (step) {
//       const stepNumber = parseInt(step, 10)
//       if (!isNaN(stepNumber) && stepNumber >= 1 && stepNumber <= 8) {
//         // Auth guard: can't go beyond step 1 without authentication
//         if (stepNumber > 1 && Option.isNone(dashboard.session)) {
//           currentSlide = 0 // Force back to step 1
//         } else {
//           currentSlide = stepNumber - 1 // Convert to 0-based index
//         }
//       }
//     }
//     isInitialized = true
//   }
// })

// // Update URL when slide changes (but not during initialization)
// $effect(() => {
//   if (browser && isInitialized && currentSlide >= 0) {
//     const currentStep = page.url.searchParams.get("step")
//     const newStep = (currentSlide + 1).toString()

//     // Only update if the URL step is different from current slide
//     if (currentStep !== newStep) {
//       const url = new URL(page.url)
//       url.searchParams.set("step", newStep)
//       goto(url.toString(), { replaceState: true, noScroll: true })
//     }
//   }
// })

// function goToNextSlide() {
//   // Auth guard: can't proceed beyond step 1 without authentication
//   if (currentSlide === 0 && Option.isNone(dashboard.session)) {
//     return // Don't proceed if not authenticated
//   }
//   stepperCardRef.goToNextSlide()
// }

// function goToSlide(index: number) {
//   // Auth guard: can't go beyond step 1 without authentication
//   if (index > 0 && Option.isNone(dashboard.session)) {
//     return // Don't proceed if not authenticated
//   }
//   stepperCardRef.goToSlide(index)
// }

// function goToPreviousSlide() {
//   stepperCardRef.goToPreviousSlide()
// }
</script>

<!-- <div class="flex justify-center items-center h-full w-full">
  <StepperCard
    bind:this={stepperCardRef}
    bind:currentSlide
    totalSlides={8}
    class="max-w-5xl md:h-auto"
  >
    {#snippet children(slideIndex)}
      <div class="flex flex-col gap-4 h-full w-full">
        {#if slideIndex === 0}
          <Step1 onNext={goToNextSlide} />
        {:else if slideIndex === 1}
          <Step2
            onNext={goToNextSlide}
            onBack={goToPreviousSlide}
          />
        {:else if slideIndex === 2}
          <Step3
            onNext={goToNextSlide}
            onBack={goToPreviousSlide}
          />
        {:else if slideIndex === 3}
          <Step4
            onNext={goToNextSlide}
            onSkip={goToNextSlide}
            onBack={goToPreviousSlide}
          />
        {:else if slideIndex === 4}
          <Step5
            onNext={goToNextSlide}
            onSkip={goToNextSlide}
            onBack={goToPreviousSlide}
          />
        {:else if slideIndex === 5}
          <Step6
            onNext={goToNextSlide}
            onBack={goToPreviousSlide}
          />
        {:else if slideIndex === 6}
          <Step7
            onNext={goToNextSlide}
            onBack={goToPreviousSlide}
          />
        {:else if slideIndex === 7}
          <Step8
            onRestart={() => goToSlide(0)}
            onBack={goToPreviousSlide}
          />
        {/if}
      </div>
    {/snippet}
  </StepperCard>
</div> -->
