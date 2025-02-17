<script lang="ts">
import TracesSwitch from "$lib/components/DevSettings/TracesSwitch.svelte"
import DetailsSwitch from "$lib/components/DevSettings/DetailsSwitch.svelte"
</script>

<div class="flex flex-col space-y-2 mt-auto">
  <h3 class="uppercase font-supermolot font-bold text-xl">Developer settings</h3>
  <div class="flex flex-col space-y-2">
    <TracesSwitch />
    <DetailsSwitch />
  </div>
</div>