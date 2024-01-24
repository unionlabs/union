<script lang="ts">
  import '#/styles/index.css'
  import * as d3 from 'd3'
  import { onMount } from 'svelte'
  import * as Plot from '@observablehq/plot'
  import { roundNumber } from '#/lib/utilities.ts'

  export const [constraintsLineColor, ramLineColor] = ['#3F8EF7', '#9DA3AE']

  const pauseAnimation = (element: SVGPathElement) => (element.style.animationPlayState = 'paused')
  const resumeAnimation = (element: SVGPathElement) =>
    (element.style.animationPlayState = 'running')

  function getRelevantPathElements({ selector }: { selector: string }) {
    const gElements = document.querySelectorAll(selector)
    const pathElements = Array.from(gElements)
      .map(gElement => gElement.querySelector('path') as SVGPathElement)
      .filter(Boolean)
    return pathElements
  }

  let chartElement: HTMLElement

  const i = d3.interpolateNumber(0, 100);
  const interpolate = (n: number) => {
    console.log(n, i(n))
    return i(n)
  }

  // https://unionlabs.github.io/galois-benchmark/c6i.x32large.v3/report.html
  let constraints = [
    { x: 4, y: 2650000 },
    { x: 8, y: 2680645 },
    { x: 16, y: 2741935 },
    { x: 32, y: 2864516 },
    { x: 64, y: 3109677 },
    { x: 128, y: 3600000 },
  ]

  const constraintsPoints = constraints.map(({x, y}) => {return {x: x, y: interpolate(y / 3600000)}})

  let ram = [
    { x: 4, y: 2.9 },
    { x: 8, y: 3.06 },
    { x: 16, y: 3.4 },
    { x: 32, y: 4.05 },
    { x: 64, y: 5.3 },
    { x: 128, y: 8 },
  ]

  const ramPoints = ram.map(({x, y}) => {return {x: x, y: interpolate(y / 8)}})

  const points = [
    ...constraintsPoints.map(r => ({x: r.x, y: r.y, z: "constraints", stroke: constraintsLineColor})),
    ...ramPoints.map(r => ({x: r.x, y: r.y, z: "ram", stroke: ramLineColor}))
  ]

  const xyz = {
    x: 'x',
    y: 'y',
    z: 'z',
    stroke: 'stroke',
  }

  $: {
    chartElement?.append(
      // @ts-expect-error
      Plot.plot({
        className: "galois-graph",
        style: {
          borderRadius: '5px',
          fontFamily: 'monospace',
          fontVariantNumeric: 'tabular-nums'
        },
        x: {
          domain: [-5, 140],
          tickSize: 0,
          axis: 'bottom',
          labelAnchor: 'center',
          legend: true,
          tickFormat: (d: number) => d
        },
        y: {
          domain: [0, 200],
          axis: null
        },
        figure: true,
        marks: [
          Plot.gridY({ stroke: '#1f1f1f', strokeWidth: 1, strokeOpacity: 1 }),
          Plot.ruleY([1], { stroke: '#1f1f1f', strokeWidth: 1 }),
          Plot.line(points, {
            ...xyz,
            markerStart: 'none',
            strokeWidth: 3,
            curve: 'linear',
          }),
          // Plot.line(ramPoints, {
          //   markerStart: 'none',
          //   x: 'x',
          //   y: 'y',
          //   strokeWidth: 3,
          //   curve: 'linear',
          //   stroke: ramLineColor
          // }),
          Plot.dot(points, {
            ...xyz,
            strokeWidth: 3,
          }),
          // Plot.tip(points, Plot.pointerX({
          //   ...xyz,
          //   strokeWidth: 3,
          // })),
          Plot.ruleX(points, Plot.pointerX({ x: 'x', py: 'y', stroke: "#1f1f1f" })),

          Plot.tip(
            points,
            Plot.pointerX(Plot.groupX({
                y: 'y',
                title: ({ x, y }) => `constraints ${x}\n\n𝑦 ${roundNumber(y, 2)}s`
              },
              {
                x: 'x',
                y: 'y',
                z: 'x',
            })
          ))
        ]
      })
    )
  }

  onMount(() => {
    const observer = new IntersectionObserver(
      entries => {
        // const pathElements = getRelevantPathElements({
        //   selector: `g[stroke="${tendermindXLineColor}"], g[stroke="${galoisLineColor}"]`
        // })
        // // const pathElementsLengths = pathElements.map(pathElement => pathElement.getTotalLength())
        // entries.forEach(entry => {
        //   console.log(entry.isIntersecting)
        //   if (entry.isIntersecting) pathElements.forEach(resumeAnimation)
        //   else pathElements.forEach(pauseAnimation)
        // })
      },
      { threshold: 0.5 }
    )

    observer.observe(chartElement)
  })
</script>

<div class="w-full text-center flex antialiased">
  <!-- <p

    class="transform rotate-180 text-md sm:text-xl font-semibold absolute md:-left-32 -left-18 my-auto mx-auto h-[75%]"
    style="writing-mode: vertical-lr;"
    id="y-axis-label"
  >
    Seconds to prove
  </p> -->
  <div>
    <article
      data-graph="performance"
      bind:this={chartElement}
    ></article>
  </div>
</div>

<style>
  /* animation: line-progress 2s linear infinite normal forwards running; */
  /* :root {
    --animation-direction: normal;
    --animation-play-state: running;
    --animation-timing-function: ease;
    --animation-iteration-count: 1;
    --animation-fill-mode: forwards;
    --axis-tick-label-font-size: 12px;
    --axis-label-font-size: 1rem;
    --axis-label-color: transparent;
  }

  article figure svg {
    -webkit-font-feature-settings: 'c2cs';
  }

  :global(g[aria-label='tip'] g path) {
    border-radius: 50px !important;
    padding: 0.5rem !important;
    background-color: red !important;
  }

  :global(article figure svg) {
  }

  :global(article figure svg g[aria-label='y-grid'] line) {
    stroke: rgb(113, 113, 113);
    -webkit-background-clip: text !important;
    background-clip: text !important;
    -webkit-text-fill-color: transparent !important;
  }

  :global(figure > svg) {
    scale: 1.5;
    width: 100%;
  }

  :global(#y-axis-label) {
    left: -3rem;
  }

  :global(#x-axis-label) {
    position: relative;
    margin-top: 60px !important;
  }

  @media (min-width: 790px) {
  }
  @media (max-width: 790px) {
    :global(#y-axis-label) {
      margin-left: 0px !important;
      font-size: 1rem !important;
    }
  }

  @media (max-width: 1024px) {
    :global(figure > svg) {
      scale: 1.3;
    }
  }
  @media (min-width: 1024px) {
    :global(#x-axis-label) {
      margin-top: 92px !important;
    }
  }

  @media (max-width: 891px) {
    :global(figure > svg) {
      scale: 1.2;
    }
    :global(#y-axis-label) {
      left: -2rem !important;
    }

    :global(#x-axis-label) {
      margin-top: 32px !important;
    }
  }

  :global(g[aria-label='y-axis tick label'] text) {
    font-size: var(--axis-tick-label-font-size);
    font-family: var(--axis-label-font-family);
  }

  :global(g[aria-label='x-axis tick label'] text) {
    font-size: var(--axis-tick-label-font-size);
    font-family: var(--axis-label-font-family);
  }

  :global(g[aria-label='y-axis label']) {
    color: var(--axis-label-color);
    display: none;
    font-size: 0;
    visibility: hidden;
  }

  :global(g[aria-label='x-axis label']) {
    color: var(--axis-label-color);
    display: none;
  }

  :global(g[stroke='#9DA3AE'] path) {
    /*
    * to get this exact length, call `pathElement.getTotalLength()`
    /
    stroke-dasharray: 668px;
    stroke-dashoffset: 668px;
    stroke-width: 2.5px;
    animation-name: slow-line-progress;
    animation-duration: 1.5s;
    animation-direction: var(--animation-direction);
    animation-play-state: var(--animation-play-state);
    animation-fill-mode: var(--animation-fill-mode);
    animation-timing-function: var(--animation-timing-function);
    animation-iteration-count: var(--animation-iteration-count);
  }

  @keyframes slow-line-progress {
    0% {
      stroke-dashoffset: 668px;
    }
    100% {
      stroke-dashoffset: 0%;
    }
  }

  :global(g[stroke='#9DA3AE'] circle) {
    opacity: 0;
    animation-name: fade-in;
    animation-duration: 1.5s;
    animation-delay: 1.5s;
    animation-fill-mode: var(--animation-fill-mode);
  }

  @keyframes fade-in {
    0% {
      opacity: 0;
    }
    100% {
      opacity: 1;
    }
  }

  :global(g[stroke='#3F8EF7'] path) {
    /*
    * to get this exact length, call `pathElement.getTotalLength()`
    *
    stroke-dasharray: 580px;
    stroke-dashoffset: 580px;
    stroke-width: 2px;
    animation-duration: 2.5s;
    animation-name: fast-line-progress;
    animation-direction: var(--animation-direction);
    animation-play-state: var(--animation-play-state);
    animation-fill-mode: var(--animation-fill-mode);
    animation-timing-function: var(--animation-timing-function);
    animation-iteration-count: var(--animation-iteration-count);
  }

  :global(g[stroke='#3F8EF7'] circle) {
    opacity: 0;
    animation-name: fade-in;
    animation-duration: 1.5s;
    animation-delay: 1.5s;
    animation-fill-mode: var(--animation-fill-mode);
  }

  @keyframes fast-line-progress {
    0% {
      stroke-dashoffset: 580px;
    }
    100% {
      stroke-dashoffset: 0%;
    }
  } */
</style>
