<script lang="ts">
  import { onMount } from 'svelte'

  // auto refetch jsonplaceholder every 10 seconds

  let interval: NodeJS.Timer
  let results: Array<any> = []

  // onMount(() => {
  //   interval = setInterval(() => {
  //     fetch('https://jsonplaceholder.typicode.com/todos')
  //       .then(res => res.json())
  //       .then(data => {
  //         results = data
  //       })
  //   }, 10000)
  // })
</script>

<div>
  <!-- {#each results as result}
    <div>{result.title}</div>
    <div>{result.body}</div>
  {/each} -->
  <slot />
</div>
