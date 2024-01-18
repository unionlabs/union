<script lang="ts">
  import { onMount } from 'svelte'
  import { highlightCode } from '#/lib/highlight-code.ts'

  export let code: string
  let highlightedCode = ''

  onMount(async () => {
    highlightedCode = await highlightCode(code)
  })
</script>


{@html highlightedCode}


<style>
  :global(code) {
    padding: 10px 20px 20px 10px;
  }

  :global(span) {
    padding: 0px !important;
  }
</style>
