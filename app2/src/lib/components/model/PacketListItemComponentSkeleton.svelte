<script lang="ts">
import Skeleton from "$lib/components/ui/Skeleton.svelte"
</script>

<div class="flex flex-col md:flex-row gap-4 p-4 border-b border-zinc-800">
  <div class="flex flex-col gap-2 flex-1">
    <div class="flex items-center gap-2">
      <div class="text-xs text-zinc-500">From:</div>
      <Skeleton class="h-6 w-32" />
    </div>
    <div class="flex items-center gap-2">
      <div class="text-xs text-zinc-500">To:</div>
      <Skeleton class="h-6 w-32" />
    </div>
  </div>

  <div class="flex flex-col gap-2 flex-1">
    <div class="flex items-center gap-2">
      <div class="text-xs text-zinc-500">Status:</div>
      <Skeleton class="h-6 w-24" />
    </div>
    <div class="flex items-center gap-2">
      <div class="text-xs text-zinc-500">Channel:</div>
      <Skeleton class="h-6 w-24" />
    </div>
  </div>

  <div class="flex flex-col gap-2 flex-1">
    <div class="flex items-center gap-2">
      <div class="text-xs text-zinc-500">Sent:</div>
      <Skeleton class="h-6 w-32" />
    </div>
    <div class="flex items-center gap-2">
      <div class="text-xs text-zinc-500">Received:</div>
      <Skeleton class="h-6 w-32" />
    </div>
  </div>
</div>
