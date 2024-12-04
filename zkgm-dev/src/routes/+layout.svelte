<script lang="ts">
  import "../app.css"
  import Bar from "$lib/components/Bar.svelte";
  import Agents from "$lib/components/Agents.svelte";
  import Video from "$lib/components/Video.svelte";

  let {children} = $props()
</script>

{@render children()}