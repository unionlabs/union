<script>
import LoadingDots from "./loading-dots.svelte"
</script>

  <div class="flex max-h-auto overflow-hidden text-muted-foreground">
    <div class="relative w-12 h-4">
      <LoadingDots class="absolute -top-4 size-12 h-12 w-12"/>
    </div>
    <slot/>
  </div>
