<script lang="ts">
  import * as d3 from 'd3'
  import * as Plot from '@observablehq/plot'

  let div: HTMLDivElement
  let data = d3.ticks(-2, 2, 200).map(Math.sin)

  function onMouseMove(event: MouseEvent) {
    const [x, y] = d3.pointer(event)
    data = data.slice(-200).concat(Math.atan2(y, x))
  }

  $: {
    div?.firstChild?.remove() // remove old chart, if any
    // @ts-expect-error
    div?.append(Plot.lineY(data).plot({ grid: true })) // add the new chart
  }
</script>

<div
  on:mousemove={onMouseMove}
  bind:this={div}
  role="img"
></div>
