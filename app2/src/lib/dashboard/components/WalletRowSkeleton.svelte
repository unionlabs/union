<script lang="ts">
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
</script>

<div class="flex items-center justify-between gap-4 px-4 pl-6 py-3 border-t border-zinc-800">
  <!-- Potential colored group indicator bar placeholder - a div with width might be better for skeleton if we show it-->
  <!-- <Skeleton class="absolute left-0 top-0 bottom-0 w-1" /> -->
  
  <div class="flex-grow min-w-0 pr-4"><Skeleton class="h-4 w-full md:w-3/4" /></div> <!-- Address column expanded -->
  <div class="w-1/4 hidden md:block min-w-0"><Skeleton class="h-4 w-3/4" /></div> <!-- Added At column --> 
  <div class="w-auto"><Skeleton class="h-6 w-16" /></div> <!-- Action button wider --> 
</div> 