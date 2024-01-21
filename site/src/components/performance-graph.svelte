<script lang="ts">
  import { onMount } from 'svelte'
  import * as Plot from '@observablehq/plot'
  import { roundNumber } from '#/lib/utilities.ts'

  export const [zkFastLineColor, zkSlowLineColor] = ['#1ED2FA', '#9DA3AE']

  export const generateRandomNumber = (min: number, max: number) =>
    Math.random() * (max - min) + min

  export const pauseAnimation = (element: SVGPathElement) =>
    (element.style.animationPlayState = 'paused')
  export const resumeAnimation = (element: SVGPathElement) =>
    (element.style.animationPlayState = 'running')

  export function getRelevantPathElements({ selector }: { selector: string }) {
    const gElements = document.querySelectorAll(selector)
    const pathElements = Array.from(gElements).map(
      gElement => gElement.querySelector('path') as SVGPathElement
    )
    return pathElements
  }

  /**
   * Intersection Observer
   */
  export function intersectionObserver({
    onIntersectCallback,
    onUnintersectCallback
  }: {
    onIntersectCallback: () => void
    onUnintersectCallback: () => void
  }) {
    const chartElement = document.querySelector('article[data-graph="performance"]') as HTMLElement
    const observer = new IntersectionObserver(
      entries => {
        const pathElements = getRelevantPathElements({
          selector: `g[stroke="${zkSlowLineColor}"], g[stroke="${zkFastLineColor}"]`
        })
        entries.forEach(entry => {
          console.log('intersection observer', entry.isIntersecting, entry.intersectionRatio)
          if (entry.isIntersecting) pathElements.forEach(resumeAnimation)
          // else pathElements.forEach(pauseAnimation)
        })
      },
      { threshold: 0.5 }
    )
    return {
      observe: () => observer.observe(chartElement),
      unobserve: () => observer.unobserve(chartElement)
    }
  }

  let chartElement: HTMLElement
  /**
   * Plot data
   * TODO: replace with real data
   */
  const totalLength = 50
  let zkSlowPlotLine = Array.from({ length: totalLength }, (_, index) => ({
    x: index,
    y: 1 + index * 0.1
  }))
  let zkFastPlotLine = Array.from({ length: totalLength }, (_, index) => ({
    x: index,
    y: index + generateRandomNumber(-2, 2)
  }))

  $: {
    chartElement?.append(
      // @ts-expect-error
      Plot.plot({
        width: 800,
        height: 400,

        x: {
          tickSize: 0,
          axis: 'bottom',

          label: '# of validators',
          ariaLabel: '#-validators',
          tickFormat: (d: number) => d
        },
        y: {
          tickSize: 0,
          axis: 'left',
          labelAnchor: 'top',
          label: 'Seconds to prove',
          scheme: 'Viridis',
          fontVariant: 'tabular-nums',
          tickFormat: (d: number) => d,
          ariaDescription: 'seconds-to-prove'
        },
        figure: true,

        marks: [
          Plot.gridY({
            stroke: '#ffffff',
            strokeWidth: 1,
            strokeOpacity: 0.3
          }),
          Plot.gridY([0], {
            x: (y, _index) => y,
            color: '#ffffff',
            strokeWidth: 0.5,
            strokeOpacity: 0.5
          }),
          Plot.line([{ x: 0, y: 2 }].concat(zkFastPlotLine), {
            x: 'x',
            y: 'y',
            curve: 'catmull-rom',
            stroke: zkFastLineColor
          }),
          Plot.line([{ x: 0, y: 1 }].concat(zkSlowPlotLine), {
            x: 'x',
            y: 'y',
            stroke: zkSlowLineColor,
            curve: 'bump-y'
          }),
          Plot.tip(
            zkFastPlotLine,
            Plot.pointerX({
              x: 'x',
              y: 'y',
              fontSize: 12,
              fill: '#181A21',
              fillOpacity: 1,
              strokeWidth: 0,
              textAnchor: 'start',
              fontWeight: 'bolder',
              frameAnchor: 'middle',
              pointerEvents: 'none',
              fontVariant: 'tabular-nums',
              title: ({ x, y }) => `↑ ${x}\n\n→ ${roundNumber(y, 2)}s`
            })
          ),
          Plot.dot(
            zkFastPlotLine,
            Plot.pointerX({ x: 'x', y: 'y', stroke: 'red', fill: 'red', r: 3 })
          )
        ]
      })
    )
  }
  onMount(() => {
    const observer = new IntersectionObserver(
      entries => {
        const pathElements = getRelevantPathElements({
          selector: `g[stroke="${zkSlowLineColor}"], g[stroke="${zkFastLineColor}"]`
        })
        entries.forEach(entry => {
          console.log('intersection observer', entry.isIntersecting, entry.intersectionRatio)
          if (entry.isIntersecting) pathElements.forEach(resumeAnimation)
          else pathElements.forEach(pauseAnimation)
        })
      },
      { threshold: 0.5 }
    )

    observer.observe(chartElement)
  })
</script>

<article
  data-graph="performance"
  bind:this={chartElement}
></article>

<style>
  /* animation: line-progress 2s linear infinite normal forwards running; */
  :root {
    --animation-direction: normal;
    --animation-play-state: paused;
    --animation-timing-function: ease;
    --animation-iteration-count: 1;
    --axis-label-font-size: 20px;
  }

  :global(g[aria-label='y-axis label'] text) {
    font-size: var(--axis-label-font-size);
    font-family: var(--axis-label-font-family);
  }

  :global(g[aria-label='x-axis label'] text) {
    text-align: center;
    font-size: var(--axis-label-font-size);
    font-family: var(--axis-label-font-family);
  }

  :global(g[stroke='#9DA3AE'] path) {
    stroke-dasharray: 105%;
    /* stroke-dashoffset: 100%; */
    stroke-width: 2.5px;
    animation-name: slow-line-progress;
    animation-duration: 4.5s;
    animation-direction: var(--animation-direction);
    animation-play-state: var(--animation-play-state);
    animation-timing-function: var(--animation-timing-function);
    animation-iteration-count: var(--animation-iteration-count);
  }

  :global(g[stroke='#1ED2FA'] path) {
    stroke-dasharray: 180%;
    /* stroke-dashoffset: 180%; */
    stroke-width: 2px;
    animation-duration: 2.5s;
    animation-name: fast-line-progress;
    animation-direction: var(--animation-direction);
    animation-play-state: var(--animation-play-state);
    animation-timing-function: var(--animation-timing-function);
    animation-iteration-count: var(--animation-iteration-count);
  }

  @keyframes fast-line-progress {
    0% {
      stroke-dashoffset: 100%;
    }
    100% {
      stroke-dashoffset: 0%;
    }
  }
  @keyframes slow-line-progress {
    0% {
      stroke-dashoffset: 100%;
    }
    100% {
      stroke-dashoffset: 0%;
    }
  }
</style>
