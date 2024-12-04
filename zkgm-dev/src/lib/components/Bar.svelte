<script lang="ts">

  import Blink from "$lib/components/Blink.svelte";
  import Agents from "$lib/components/Agents.svelte";
</script>

<div class="fixed bottom-0 inset-x-0">
  <Agents/>
  <div class="w-full bg-black p-4 flex items-center justify-center">
    <p class="font-supermolot font-bold text-union-accent-500"><Blink /></p>
  </div>
</div>