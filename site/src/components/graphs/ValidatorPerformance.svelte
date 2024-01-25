<script lang="ts">
  import '#/styles/index.css'
  import tw from '../../../tailwind.config.ts'
  import { onMount } from 'svelte'
  import * as echarts from 'echarts';

  const constraintsLineColor = `${tw.theme.extend.colors.accent[900]}`
  const ramLineColor = `${tw.theme.extend.colors.accent[500]}`
  const timeLineColor = `${tw.theme.extend.colors.accent[700]}`

  // https://unionlabs.github.io/galois-benchmark/c6i.x32large.v3/report.html
  let constraints = [
    { x: 4, y: 2650000 },
    { x: 8, y: 2680645 },
    { x: 16, y: 2741935 },
    { x: 32, y: 2864516 },
    { x: 64, y: 3109677 },
    { x: 128, y: 3600000 },
  ]

  let ram = [
    { x: 4, y: 2.9 },
    { x: 8, y: 3.06 },
    { x: 16, y: 3.4 },
    { x: 32, y: 4.05 },
    { x: 64, y: 5.3 },
    { x: 128, y: 8 },
  ]

  let time = [
    { x: 4, y: 6.05 },
    { x: 8, y: 6.18 },
    { x: 16, y: 6.23 },
    { x: 32, y: 6.70 },
    { x: 64, y: 6.91 },
    { x: 128, y: 8.10 },
  ]

  function seriesValue(value: any, name: string): string {
    switch (name) {
      case "Ram":
        return `${value}GB`;
      case "Time":
        return `${value}s`;
      case "Constraints":
        // NOTE: We can use toLocaleString here
        return new Intl.NumberFormat('en-US', { style: 'decimal' }).format(value);
      default:
        return "";
    }
  }

  let hiddenData = Array(128 / 4).fill(0).map((_, i) => [i * 4, '-'])
  console.log(hiddenData)

  let myChart;

  onMount(() => {
    let chartDom = document.getElementById('galois-graph');
    let myChart = echarts.init(chartDom, 'light', { renderer: 'svg' });
    let option;

    option = {
      animation: true,
      animationDuration: 1500,
      tooltip: {
        trigger: 'axis',
        axisPointer: {
          // show: true,
          z: -10000,
          lineStyle: {
            color: "#1f1f1f",
            type: "solid",
          }
        },
        borderColor: "#1f1f1f",
        backgroundColor: "#000",
        formatter(params, ticket, callback) {
          return `
            <div style="margin: 0px 0 0;line-height:1;">
               <div style="font-size:14px;color:#666;font-weight:400;line-height:1;">${params[0].value[0]} Validators</div>
               <div style="margin: 10px 0 0;line-height:1;">
                 ${
                   params.map(x => `
                    <div style="margin: 0px 0 0;line-height:1;">
                       <div class="text-left" style="margin: 0px 0 0;line-height:1;">
                          ${x.marker}
                          <span class="text-left w-full" style="font-size:14px;color:#666;font-weight:400;margin-right:auto;text-align:start">${x.seriesName}</span>
                          <span class="font-mono" style="float:right;margin-left:20px;font-size:14px;color:#666">${seriesValue(x.value[1], x.seriesName)}</span>
                          <div style="clear:both"></div>
                       </div>
                       <div style="clear:both"></div>
                    </div>
                   `).join('')
                 }
               </div>
               <div style="clear:both"></div>
            </div>
          `
        }
      },
      grid: {
        left: '0%',
        right: '0%',
        bottom: '16.66666%',
        top: '-1px',
      },
      xAxis: {
        axisLine: {
          show: false
        },
        splitLine: {
          show: false
        },
        axisTick: {
          show: false
        },
        interval: 4,
        axisLabel: {
          formatter(value: number, _index: any) {
            return [4, 64, 128].includes(value) ? value : null
          },
        },
        type: 'value',
        boundaryGap: false,
        min(value: { min: number }) {
            return value.min - 12;
        },
        max(value: { max: number }) {
            return value.max + 12;
        },
      },
      yAxis: [{
        axisLine: {
          show: false
        },
        axisTick: {
          show: false
        },
        axisLabel: {
          show: false
        },
        splitLine: {
          lineStyle: {
            color: "#1f1f1f"
          }
        },
        max(value: any) {
            return 6_000_000;
        },
        interval: 1_200_000,
      },
      {
        show: false,
        max(value) {
            return value.max * (6_000_000 / 1_600_000);
        },
      },
      {
        show: false,
        max(value) {
            return value.max * (6_000_000 / 2_800_000);
        },
      }],
    };

    option && myChart.setOption(option);

    window.addEventListener('resize', function() {
      myChart.resize();
    });
      
    const observer = new IntersectionObserver(
      entries => {
        entries.forEach(entry => {
          if (entry.isIntersecting) {
            myChart.setOption({
              series: [
                {
                  name: 'Constraints',
                  type: 'line',
                  yAxisIndex: 0,
                  symbol: 'circle',
                  data: constraints.map(({x,y})=>[x,y]),
                  lineStyle: { color: constraintsLineColor },
                  itemStyle: { color: constraintsLineColor },
                },
                {
                  name: 'Time',
                  type: 'line',
                  yAxisIndex: 2,
                  symbol: 'circle',
                  data: time.map(({x,y})=>[x,y]),
                  lineStyle: { color: timeLineColor },
                  itemStyle: { color: timeLineColor },
                },
                {
                  name: 'Ram',
                  type: 'line',
                  yAxisIndex: 1,
                  symbol: 'circle',
                  data: ram.map(({x,y})=>[x,y]),
                  lineStyle: { color: ramLineColor },
                  itemStyle: { color: ramLineColor },
                },
              ]
            })
          }
        })
      },
      { threshold: 0.5 }
    )

    observer.observe(chartDom)
  })
</script>

<div class="w-full h-full text-center flex antialiased">
  <article
    id="galois-graph"
    class="w-full h-full"
    bind:this={myChart}
  ></article>
</div>
