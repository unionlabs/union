<script lang="ts">
  import Blink from "$lib/components/Blink.svelte";
</script>

<div class="w-full bg-black p-4 flex items-center justify-between">
  <img src="/logo.svg" alt="union logo" class="h-10 w-auto">
  <p class="show-430 font-supermolot font-bold text-union-accent-500">
    <Blink/>
  </p>
  <a href="https://union.build" class="font-mono font-bold text-union-accent-500">[MAIN SITE]</a>
</div>

<style lang="postcss">
  .show-430 {
    display: none;
  }

  @media screen and (min-width: 430px) {
    .show-430 {
      display: block;
    }
  }
</style>
