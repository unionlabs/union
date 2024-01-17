<script lang="ts">
  import * as d3 from 'd3'
  import { onMount } from 'svelte'
  import { fade } from 'svelte/transition'
  import * as Plot from '@observablehq/plot'
  // import { inView } from '#/lib/actions/in-view.ts'

  /**
   * @docs
   * - https://observablehq.com/plot
   * - https://d3js.org
   */

  let div: HTMLDivElement
  let visible = false
  let loading = true

  const pointsCount = 50

  const plotLineA = Array.from({ length: pointsCount }, (_, index) => ({
    x: index,
    y: index
  }))

  const plotLine = Array.from({ length: pointsCount }, (_, index) => index) as [number, number]

  const plotLineB = Array.from({ length: pointsCount }, (_, index) => ({
    x: index,
    y: index * 2
  }))

  $: {
    div?.firstChild?.remove() // remove old chart, if any
    div?.append(
      // @ts-expect-error
      Plot.plot({
        className: 'bg-black',
        caption: null,
        figure: false,
        label: null,
        marks: [
          Plot.ruleY([0]),
          Plot.lineY(plotLineA, { x: 'x', y: 'y', stroke: 'red' }),
          Plot.lineY(plotLineB, { x: 'x', y: 'y', stroke: 'blue' }),
          Plot.tip(
            plotLineA,
            Plot.pointer({
              x: 'x',
              y: 'y',
              fill: 'black',
              pointerEvents: 'none',
              title: ({ x, y }) => `${x}, ${y}`
            })
          )
          // Plot.tip(
          //   plotLineB,
          //   Plot.pointer({
          //     x: 'x',
          //     y: 'y',
          //     fill: 'black',
          //     pointerEvents: 'none',
          //     title: ({ x, y }) => `${x}, ${y}`
          //   })
          // )
        ]
      })
    ) // add the new chart
  }

  onMount(() => {
    console.log('onMount')

    // const intersectionObserver = new IntersectionObserver(
    //   (entries, _observer) => {
    //     entries.forEach(entry =>
    //       entry.target.dispatchEvent(new CustomEvent('intersect', { detail: entry }))
    //     )
    //   },
    //   {
    //     root: document.querySelector('hmtl'),
    //     rootMargin: '0px',
    //     threshold: 0
    //   }
    // )

    // const node = document.querySelector('div[data-graph-intersect]') as HTMLElement
    // intersectionObserver.observe(node)
    // node.dataset.intersectInitialized = 'true'

    // return () => {
    //   intersectionObserver.unobserve(node)
    // }
  })
</script>
<!--  -->
<!-- {#if visible} -->
  <div
    role="img"
    class="visible"
    bind:this={div}
    transition:fade={{ duration: 1200 }}
    on:intersect={event => {
      console.log(event)
      visible = event.detail.isIntersecting
    }}
  ></div>
<!-- {/if} -->
