<script lang="ts">
  import '#/styles/index.css'
  import { onMount } from 'svelte'
  import { highlightCode } from '#/lib/highlight-code.ts'

  /**
   * This code block component is used to display code snippets outside of md/mdx files.
   * The snippet has to have a language tag, e.g. ```[language] [code] ```.
   * If you're doing a multi-line string with backticks, you have to escape the backticks with a backslash.
   * @example: https://stackblitz.com/edit/github-j3wpz9?file=src%2Fpages%2Fsnippet.astro
   */

  export let code: string
  let highlightedCode = ''

  onMount(async () => (highlightedCode = await highlightCode(code)))
</script>

<div class="">
  {@html highlightedCode}
</div>

<style>
  :global(.container > figure[data-rehype-pretty-code-figure] span[data-line]) {
    overflow-x: scroll;
    max-width: 100%px;
    width: 100%;
    font-size: 0.75rem; /* 12px */
    line-height: 1rem; /* 16px */
  }

  @media (max-width: 400px) {
    :global(figure[data-rehype-pretty-code-figure]) {
      max-width: 385px;
    }
  }
</style>
