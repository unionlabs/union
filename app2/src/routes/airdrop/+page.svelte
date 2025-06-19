<script lang="ts">
  import Card from '$lib/components/ui/Card.svelte';
  import Sections from '$lib/components/ui/Sections.svelte';
  import SlideContent from './SlideContent.svelte';
  import Button from '$lib/components/ui/Button.svelte';

  let currentSlide = $state(0)
  let slideCardRef: SlideContent
</script>

<Sections class="flex items-center w-full h-full">
  <Card class="w-full h-full">
    <SlideContent
      bind:this={slideCardRef}
      bind:currentSlide
      totalSlides={2}
      class="w-full h-full"
    >
      {#snippet children(slideIndex)}
        <div class="flex flex-col p-4 text-center gap-6">
          {#if slideIndex === 0}
            <h1 class="text-4xl font-bold">Page 1</h1>
            <p class="text-gray-600">This is the first page</p>
            <Button onclick={() => slideCardRef.goToNextSlide()}>Next</Button>
          {:else if slideIndex === 1}
            <h1 class="text-4xl font-bold">Page 2</h1>
            <p class="text-gray-600">This is the second page</p>
            <Button onclick={() => slideCardRef.goToPreviousSlide()}>Previous</Button>
          {/if}
        </div>
      {/snippet}
    </SlideContent>

  </Card> 
</Sections>
