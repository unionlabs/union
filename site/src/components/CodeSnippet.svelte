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
  /* :global(ul[role="tablist"]){
    border-bottom: 2px solid #1f1f1f;
  }

  :global(a:not([tabindex])){
    font-family: monospace;
  padding-left: 7px !important;
  border: 1px solid #1f1f1f;
  border-color: #050505 !important;
  }

  :global(a[tabindex="-1"]){
    
  border: 0px !important;
  }

:global(a[role="tab"]){
    padding: 0 6px 0 7px !important;
  margin-right: 10px;
  margin-left: 0px;
  font-family: monospace;
  }
  :global(a[role="tab"]::before){
    content: url(https://upload.wikimedia.org/wikipedia/commons/9/98/Solidity_logo.svg);
  display: inline-block;
    width: 8px;
  height:8px;
  filter: invert(1);
  
  } */
</style>
