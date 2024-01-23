<script lang="ts">
	import * as echarts from 'echarts';
  import { onMount } from 'svelte'

	onMount(() => {
		var chartDom = document.getElementById('chart');
		var myChart = echarts.init(chartDom, null, {
		  renderer: 'svg'
		});
		var option;

		option = {
		  xAxis: {
		    type: 'category',
		    data: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']
		  },
		  yAxis: {
		    type: 'value'
		  },
		  series: [
		    {
		      data: [150, 230, 224, 218, 135, 147, 260],
		      type: 'line'
		    }
		  ]
		};

		option && myChart.setOption(option);
	})
</script>

<div class="w-full">
	<div id="chart" class="w-full"></div>
</div>

<style>
  .app {
    width: 100vw;
    height: 100vh;
  }
</style>
