<script lang="ts">
  export let title: string
  export let date: string
  export let author: string
  export let preview: string
  export let published: boolean
</script>

<div class="flex flex-1 justify-center">
  <main class="blog-post-container sm:text-xl p-4 sm:mt-4 justify-center max-w-2xl self-center">
    {#if published}
      <h1 class="text-3xl sm:text-5xl font-bold mb-0">{title}</h1>
      <div class="font-mono mb-2 mt-1 sm:text-lg">
        {date} - <a class="text-accent" href={`https://x.com/${author}`}>@{author}</a>
      </div>
      <slot />
    {/if}
  </main>
</div>
