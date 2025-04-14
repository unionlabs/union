<script lang="ts">
  import Skeleton from "$lib/components/ui/Skeleton.svelte"
</script>

<div class="flex justify-between gap-8 px-4 py-3 h-16 items-center border-b border-zinc-800">
  <div>
    <div>
      <Skeleton class="h-5 w-64" />
    </div>
    <div class="flex items-center gap-1 mt-1">
      <Skeleton class="h-4 w-24" />
      <Skeleton class="h-4 w-5" />
      <Skeleton class="h-4 w-24" />
    </div>
  </div>
  <Skeleton class="h-5 w-24" />
  <Skeleton class="h-5 w-32" />
</div>
