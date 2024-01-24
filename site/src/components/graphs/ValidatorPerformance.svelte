<script lang="ts">
  import '#/styles/index.css'
  import * as d3 from 'd3'
  import { onMount } from 'svelte'
  import * as Plot from '@observablehq/plot'
  import { roundNumber } from '#/lib/utilities.ts'

  export const [galoisLineColor, tendermindXLineColor] = ['#3F8EF7', '#9DA3AE']

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

  // https://unionlabs.github.io/galois-benchmark/c6i.x32large.v3/report.html
  let galoisExtrapolated = d3.scaleLinear([4, 128], [6.05, 8.1])
  let galois = [
    { x: 4, y: 6.05 },
    { x: 8, y: 6.18 },
    { x: 16, y: 6.23 },
    { x: 32, y: 6.94 },
    { x: 64, y: 6.91 },
    { x: 128, y: 8.1 },
    { x: 150, y: galoisExtrapolated(150) }
  ]

  // https://blog.succinct.xyz/tendermintx/
  let tendermintXExtrapolated = d3.scaleLinear([60, 150], [300, 720])
  let tendermintX = [
    { x: 4, y: tendermintXExtrapolated(4) },
    { x: 60, y: 300 },
    { x: 100, y: 480 },
    { x: 150, y: 720 }
  ]

  $: {
    chartElement?.append(
      // @ts-expect-error
      Plot.plot({
        style: {
          borderRadius: '5px',
          fontFamily: 'monospace',
          fontVariantNumeric: 'tabular-nums'
        },
        x: {
          tickSize: 0,
          axis: 'bottom',
          labelAnchor: 'center',
          ariaLabel: '#-validators',
          legend: true,
          tickFormat: (d: number) => d
        },
        y: {
          tickSize: 0,
          axis: 'left',
          legend: true,
          label: '',
          labelAnchor: undefined,
          fontVariant: 'tabular-nums',
          tickFormat: (d: number) => d,
          ariaDescription: 'seconds-to-prove'
        },
        figure: true,
        marks: [
          Plot.gridY({ strokeWidth: 1.2, strokeOpacity: 0.5 }),
          Plot.ruleY([1], { stroke: '#ffffff', strokeWidth: 1.2, strokeOpacity: 0.5 }),
          Plot.line(galois, {
            markerStart: 'none',
            x: 'x',
            y: 'y',
            strokeWidth: 3,
            curve: 'linear',
            stroke: galoisLineColor
          }),
          Plot.dot(galois, {
            x: 'x',
            y: 'y',
            strokeWidth: 3,
            stroke: galoisLineColor
          }),
          Plot.tip(
            galois,
            Plot.pointerX({
              x: 'x',
              y: 'y',
              fontSize: 16,
              stroke: '#2D2D2D',
              fill: 'rgb(24, 26, 33)',
              fillOpacity: 1,
              strokeWidth: 1,
              textAnchor: 'start',
              fontWeight: 'bolder',
              frameAnchor: 'middle',
              pointerEvents: 'none',
              fontVariant: 'tabular-nums',
              fontFamily: 'monospace',
              title: ({ x, y }) => `𝑥 ${x}\n\n𝑦 ${roundNumber(y, 2)}s`
            })
          )
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
    <p
      class="text-xs sm:text-xl font-semibold"
      id="x-axis-label"
    >
    </p>
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
