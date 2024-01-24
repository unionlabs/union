<script lang="ts">
  import '#/styles/index.css'
  import * as d3 from 'd3'
  import { onMount } from 'svelte'
  import * as Plot from '@observablehq/plot'
  import * as htl from 'htl'
  import { roundNumber } from '#/lib/utilities.ts'
  import * as echarts from 'echarts';


  // export const [constraintsLineColor, ramLineColor] = ['url(#constraints-gradient)', 'url(#ram-gradient)']
  export const [constraintsLineColor, ramLineColor] = ['#A0ECFD', '#A0ECFD']

  const pauseAnimation = (element: SVGPathElement) => (element.style.animationPlayState = 'paused')
  const resumeAnimation = (element: SVGPathElement) =>
    (element.style.animationPlayState = 'running')

  function getRelevantPathElements({ selector }: { selector: string }) {
    console.log(selector)
    const gElements = document.querySelectorAll(selector)
    const pathElements = Array.from(gElements)
      .map(gElement => gElement as SVGPathElement)
    console.log(pathElements)
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

  onMount(() => {
    let chartDom = document.getElementById('galois-graph');
    let myChart = echarts.init(chartDom, 'light', { renderer: 'svg' });
    let option;

    option = {
      title: {
      },
      tooltip: {
        trigger: 'axis'
      },
      grid: {
        left: '3%',
        right: '3%',
        bottom: '10%',
        top: '10%',
        containLabel: false,
        // borderColor: "#1f1f1f"
      },
      toolbox: {
        feature: {
          // saveAsImage: {}
        }
      },
      xAxis: {
        axisLine: {
          show: false
        },
        axisTick: {
          show: false
        },
        type: 'category',
        boundaryGap: false,
        // data: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']
      },
      yAxis: [{
        axisLine: {
          show: false
        },
        // axisTick: {
        //   show: false
        // },
        axisLabel: {
          show: false
        },
        splitLine: {
          lineStyle: {
            color: "#1f1f1f"
          }
        }
      },
      {
        axisLine: {
          show: false
        },
        // axisTick: {
        //   show: false
        // },
        axisLabel: {
          show: false
        },
        splitLine: {
          lineStyle: {
            color: "#1f1f1f"
          }
        }
      }],
      series: [
        {
          name: 'Constraints',
          type: 'line',
          yAxisIndex: 0,
          data: constraints.map(({x,y})=>[x,y])
        },
        {
          name: 'Ram',
          type: 'line',
          yAxisIndex: 1,
          data: ram.map(({x,y})=>[x,y])
        },
      ]
    };

    option && myChart.setOption(option);

    window.addEventListener('resize', function() {
      myChart.resize();
    });
      
    const observer = new IntersectionObserver(
      entries => {
        const pathElements = getRelevantPathElements({
          selector: `.galois-graph > g > path[stroke="${ramLineColor}"]`
        })
        // const pathElementsLengths = pathElements.map(pathElement => pathElement.getTotalLength())
        entries.forEach(entry => {
          console.log(entry.isIntersecting)
          if (entry.isIntersecting) pathElements.forEach(resumeAnimation)
          else pathElements.forEach(pauseAnimation)
        })
      },
      { threshold: 0.5 }
    )

    observer.observe(chartElement)
  })
</script>

<div class="w-full h-full text-center flex antialiased">
  <!-- <p

    class="transform rotate-180 text-md sm:text-xl font-semibold absolute md:-left-32 -left-18 my-auto mx-auto h-[75%]"
    style="writing-mode: vertical-lr;"
    id="y-axis-label"
  >
    Seconds to prove
  </p> -->
    <article
      id="galois-graph"
      class="w-full h-full text-center flex antialiased"
      bind:this={chartElement}
    ></article>
</div>
