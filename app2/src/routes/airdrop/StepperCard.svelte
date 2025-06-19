<script lang="ts">
  import Card from '$lib/components/ui/Card.svelte';
  import SlideContent from './SlideContent.svelte';
  import StepProgressBar from '$lib/components/ui/StepProgressBar.svelte';

  interface Props {
    currentSlide?: number;
    totalSlides: number;
    class?: string;
    children: (slideIndex: number) => any;
  }

  let { 
    currentSlide = $bindable(0), 
    totalSlides, 
    class: className = "", 
    children 
  }: Props = $props();

  let slideCardRef: SlideContent;

function goToNextSlide() {
    slideCardRef?.goToNextSlide();
  }

function goToPreviousSlide() {
    slideCardRef?.goToPreviousSlide();
  }

function goToSlide(index: number) {
    slideCardRef?.goToSlide(index);
  }

  export { goToSlide, goToNextSlide, goToPreviousSlide }
</script>

<Card class="w-full p-0 {className}">
  <StepProgressBar currentStep={currentSlide + 1} totalSteps={totalSlides} />
  <SlideContent
    bind:this={slideCardRef}
    bind:currentSlide
    {totalSlides}
    class="w-full"
  >
    {#snippet children(slideIndex)}
      {@render children(slideIndex)}
    {/snippet}
  </SlideContent>
</Card>
