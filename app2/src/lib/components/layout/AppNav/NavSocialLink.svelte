<script lang="ts">
  let { name, href }: { name: string; href: string } = $props()
</script>

<a 
  {href} 
  target="_blank" 
  rel="noreferrer" 
  class="flex items-center justify-center w-6 h-6 transition-colors duration-200 text-zinc-400 hover:text-zinc-200"
  title={name}
>
  <div class="w-4 h-4 flex items-center justify-center">
    <slot />
  </div>
</a>
