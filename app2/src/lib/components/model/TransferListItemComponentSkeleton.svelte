<script lang="ts">
import Skeleton from "../ui/Skeleton.svelte"
</script>

<div class="flex gap-8 px-4 py-2 h-[60px] items-center">
  <div class="flex-1">
    <Skeleton class="h-4 w-32 mb-2" randomWidth />
    <div class="flex items-center gap-2">
      <Skeleton class="h-4 w-24" />
      <Skeleton class="size-4" />
      <Skeleton class="h-4 w-24" />
    </div>
  </div>
  <div>
    <Skeleton class="h-4 w-30" />
  </div>
</div>
