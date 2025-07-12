<script lang="ts">
let { expanded }: { expanded: boolean } = $props()
</script>

<a
  href="/"
  class="inline-flex items-center overflow-hidden"
>
  {#if expanded}
    <img
      class="h-12 w-48 p-[3.5px] pb-[1px] shrink-0 object-left object-contain"
      src="/images/union-logo.svg"
      alt="Union"
    />
  {:else}
    <img
      class="h-12 w-48 shrink-0 object-left object-contain"
      src="/images/union-logo-glyph.svg"
      alt="Union"
    />
  {/if}
</a>
