<script lang="ts">
  import Card from "$lib/components/ui/Card.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
</script>

<Card class="flex flex-col gap-3 p-4">
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-3">
      <Skeleton class="w-6 h-6 rounded-full" />
      <Skeleton class="h-5 w-24" /> 
    </div>
    <Skeleton class="h-5 w-8" /> 
  </div>
  <Skeleton class="h-4 w-full" /> 
  <Skeleton class="h-3 w-32 mt-1" />
</Card> 