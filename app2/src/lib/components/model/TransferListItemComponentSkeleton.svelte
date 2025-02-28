<script lang="ts">
  import Label from "../ui/Label.svelte"
  import Skeleton from "../ui/Skeleton.svelte"
</script>

<div class="flex gap-8 px-4 py-2 h-[60px]">
  <div class="flex-1">
    <Label>from</Label>
    <Skeleton class="h-4" randomWidth />
  </div>
  <div class="flex-1">
    <Label>to</Label>
    <Skeleton class="h-4" randomWidth />
  </div>
  <div class="flex-1">
    <Label>Time</Label>
    <Skeleton class="h-4 w-32" />
  </div>
</div>
