// TODO(travers): support multiple time-seriies on the summary chart, once we
// have data available.
const writeThroughputWorkload = "write/values=1024";

/*
 * Returns the full URL to the write-throughput summary JSON file.
 */
function writeThroughputSummaryURL() {
    return "https://pebble-benchmarks.s3.amazonaws.com/write-throughput/summary.json";
}

/*
 * Returns the full URL to a write-throughput summary detail file, given the
 * filename.
 */
function writeThroughputDetailURL(filename) {
    return `https://pebble-benchmarks.s3.amazonaws.com/write-throughput/${filename}`;
}

/*
 * Renders the appropriate detail view given the array of data and the date
 * extract.
 *
 * This function works by using the provided date to "bisect" into the data
 * array and pull out the corresponding datapoint.
 */
function bisectAndRenderWriteThroughputDetail(data, detailDate) {
    const bisect = d3.bisector(d => parseTime(d.date)).left;
    let i = bisect(data, detailDate, 1);

    let workload = data[i];
    let date = workload.date;
    let name = workload.name;
    let opsSec = workload.opsSec;
    let filename = workload.summaryPath;

    fetchWriteThroughputSummaryData(filename)
      .then(
        d => renderWriteThroughputSummaryDetail(name, date, opsSec, d),
        _ => renderWriteThroughputSummaryDetail(name, date, opsSec, null),
      );
}

/*
 * Renders the write-throughput summary view, given the correspnding data.
 *
 * This function generates a time-series similar to the YCSB benchmark data.
 * The x-axis represents the day on which the becnhmark was run, and the y-axis
 * represents the calculated "max sustainable throughput" in ops-second.
 *
 * Clicking on an individual day renders the detail view for the given day,
 * allowing the user to drill down into the per-worker performance.
 */
function renderWriteThroughputSummary(allData) {
    const svg = d3.select(".chart.write-throughput");

    // Filter on the appropriate time-series.
    const dataKey = "write/values=1024";
    const data = allData[dataKey];

    // Set up axes.

    const margin = {top: 25, right: 60, bottom: 25, left: 60};
    let maxY = d3.max(data, d => d.opsSec);

    const width = styleWidth(svg) - margin.left - margin.right;
    const height = styleHeight(svg) - margin.top - margin.bottom;

    const x = d3.scaleTime()
        .domain([minDate, max.date])
        .range([0, width]);
    const x2 = d3.scaleTime()
        .domain([minDate, max.date])
        .range([0, width]);

    const y = d3.scaleLinear()
        .domain([0, maxY * 1.1])
        .range([height, 0]);

    const z = d3.scaleOrdinal(d3.schemeCategory10);

    const xAxis = d3.axisBottom(x)
        .ticks(5);

    const yAxis = d3.axisLeft(y)
        .ticks(5);

    const g = svg
        .append("g")
        .attr("transform", "translate(" + margin.left + "," + margin.top + ")");

    g.append("g")
        .attr("class", "axis axis--x")
        .attr("transform", "translate(0," + height + ")")
        .call(xAxis);

    g.append("g")
        .attr("class", "axis axis--y")
        .call(yAxis);

    g.append("text")
        .attr("class", "chart-title")
        .attr("x", margin.left + width / 2)
        .attr("y", 0)
        .style("text-anchor", "middle")
        .style("font", "8pt sans-serif")
        .text(dataKey);

    // Create a rectangle that can be used to clip the data. This avoids having
    // the time-series spill across the y-axis when panning and zooming.

    const defs = svg.append("defs");

    defs.append("clipPath")
        .attr("id", dataKey)
        .append("rect")
        .attr("x", 0)
        .attr("y", -margin.top)
        .attr("width", width)
        .attr("height", margin.top + height + 10);

    // Plot time-series.

    const view = g.append("g")
        .attr("class", "view")
        .attr("clip-path", "url(#" + dataKey + ")");

    const line = d3.line()
        .x(d => x(parseTime(d.date)))
        .y(d => y(d.opsSec));

    const path = view.selectAll(".line1")
        .data([data])
        .enter()
        .append("path")
        .attr("class", "line1")
        .attr("d", line)
        .style("stroke", z(0));

    // Hover to show labels.

    const lineHover = g
        .append("line")
        .attr("class", "hover")
        .style("fill", "none")
        .style("stroke", "#f99")
        .style("stroke-width", "1px");

    const dateHover = g
        .append("text")
        .attr("class", "hover")
        .attr("fill", "#f22")
        .attr("text-anchor", "middle")
        .attr("alignment-baseline", "hanging")
        .attr("transform", "translate(0, 0)");

    const opsHover = g
        .append("text")
        .attr("class", "hover")
        .attr("fill", "#f22")
        .attr("text-anchor", "middle")
        .attr("transform", "translate(0, 0)");

    const marker = g
        .append("circle")
        .attr("class", "hover")
        .attr("r", 3)
        .style("opacity", "0")
        .style("stroke", "#f22")
        .style("fill", "#f22");

    svg.node().updateMouse = function (mouse, date, hover) {
        const mousex = mouse[0];
        const bisect = d3.bisector(d => parseTime(d.date)).left;
        const i = bisect(data, date, 1);
        const v =
            i === data.length
                ? data[i - 1]
                : mousex - x(parseTime(data[i - 1].date)) < x(parseTime(data[i].date)) - mousex
                    ? data[i - 1]
                    : data[i];
        const noData = mousex < x(parseTime(data[0].date));

        let lineY = height;
        if (!noData) {
            lineY = pathGetY(path.node(), mousex);
        }

        let val, valY, valFormat;
        val = v.opsSec;
        valY = y(val);
        valFormat = d3.format(",.0f");

        lineHover
            .attr("x1", mousex)
            .attr("x2", mousex)
            .attr("y1", lineY)
            .attr("y2", height);
        marker.attr("transform", "translate(" + x(parseTime(v.date)) + "," + valY + ")");
        dateHover
            .attr("transform", "translate(" + mousex + "," + (height + 8) + ")")
            .text(formatTime(date));
        opsHover
            .attr("transform", "translate(" + x(parseTime(v.date)) + "," + (valY - 7) + ")")
            .text(valFormat(val));
    };

    // Panning and zooming.

    const updateZoom = function (t) {
        x.domain(t.rescaleX(x2).domain());
        g.select(".axis--x").call(xAxis);
        g.selectAll(".line1").attr("d", line);
    };
    svg.node().updateZoom = updateZoom;

    const zoom = d3.zoom()
        .extent([[0, 0], [width, 1]])
        .scaleExtent([0.25, 2])                         // [45, 360] days
        .translateExtent([[-width * 3, 0], [width, 1]]) // [today-360, today]
        .on("zoom", function () {
            const t = d3.event.transform;
            if (!d3.event.sourceEvent) {
                updateZoom(t);
                return;
            }

            d3.selectAll(".chart").each(function () {
                if (this.updateZoom != null) {
                    this.updateZoom(t);
                }
            });

            d3.selectAll(".chart").each(function () {
                this.__zoom = t.translate(0, 0);
            });
        });

    svg.call(zoom);
    svg.call(zoom.transform, d3.zoomTransform(svg.node()));

    svg.append("rect")
        .attr("class", "mouse")
        .attr("cursor", "move")
        .attr("fill", "none")
        .attr("pointer-events", "all")
        .attr("width", width)
        .attr("height", height + margin.top + margin.bottom)
        .attr("transform", "translate(" + margin.left + "," + 0 + ")")
        .on("mousemove", function () {
            const mouse = d3.mouse(this);
            const date = x.invert(mouse[0]);

            d3.selectAll(".chart").each(function () {
                if (this.updateMouse != null) {
                    this.updateMouse(mouse, date, 1);
                }
            });
        })
        .on("mouseover", function () {
            d3.selectAll(".chart")
                .selectAll(".hover")
                .style("opacity", 1.0);
        })
        .on("mouseout", function () {
            d3.selectAll(".chart")
                .selectAll(".hover")
                .style("opacity", 0);
        })
        .on("click", function(d) {
            // Use the date corresponding to the clicked data point to bisect
            // into the workload data to pluck out the correct datapoint.
            const mouse = d3.mouse(this);
            let detailDate = d3.timeDay.floor(x.invert(mouse[0]));
            bisectAndRenderWriteThroughputDetail(data, detailDate);
        });
}

function fetchWriteThroughputSummaryData(file) {
    return fetch(writeThroughputDetailURL(file))
      .then(response => response.json())
      .then(data => {
        for (let key in data) {
          let csvData = data[key].rawData;
          data[key].data = d3.csvParseRows(csvData, function (d, i) {
            return {
              elapsed: +d[0],
              opsSec: +d[1],
              passed: d[2] === 'true',
              size: +d[3],
              levels: +d[4],
            };
          });
          delete data[key].rawData;
        }
        return data;
      });
}

/*
 * Renders the write-throughput detail view, given the correspnding data, and
 * the particular workload and date on which it was run.
 *
 * This function generates a series with the x-axis representing the elapsed
 * time since the start of the benchmark, and the measured write load at that
 * point in time (in ops/second). Each series is a worker that participated in
 * the benchmark on the selected date.
 */
function renderWriteThroughputSummaryDetail(workload, date, opsSec, rawData) {
    const svg = d3.select(".chart.write-throughput-detail");

    // Remove anything that was previously on the canvas. This ensures that a
    // user clicking multiple times does not keep adding data to the canvas.
    svg.selectAll("*").remove();

    const margin = {top: 25, right: 60, bottom: 25, left: 60};
    let maxX = 0;
    let maxY = 0;
    for (let key in rawData) {
        let run = rawData[key];
        maxX = Math.max(maxX, d3.max(run.data, d => d.elapsed));
        maxY = Math.max(maxY, d3.max(run.data, d => d.opsSec));
    }

    const width = styleWidth(svg) - margin.left - margin.right;
    const height = styleHeight(svg) - margin.top - margin.bottom;

    // Panning and zooming.
    // These callbacks are defined as they are called from the panning /
    // zooming functions elsewhere, however, they are simply no-ops on this
    // chart, as they x-axis is a measure of "elapsed time" rather than a date.

    svg.node().updateMouse = function (mouse, date, hover) {}
    svg.node().updateZoom = function () {};

    // Set up axes.

    const x = d3.scaleLinear()
        .domain([0, 8.5 * 3600])
        .range([0, width]);

    const y = d3.scaleLinear()
        .domain([0, maxY * 1.1])
        .range([height, 0]);

    const z = d3.scaleOrdinal(d3.schemeCategory10);

    const xAxis = d3.axisBottom(x)
        .ticks(5)
        .tickFormat(d => Math.floor(d / 3600) + "h");

    const yAxis = d3.axisLeft(y)
        .ticks(5);

    const g = svg
        .append("g")
        .attr("transform", "translate(" + margin.left + "," + margin.top + ")");

    g.append("g")
        .attr("class", "axis axis--x")
        .attr("transform", "translate(0," + height + ")")
        .call(xAxis);

    g.append("g")
        .attr("class", "axis axis--y")
        .call(yAxis);

    // If we get no data, we just render an empty chart.
    if (rawData == null) {
      g.append("text")
          .attr("class", "chart-title")
          .attr("x", margin.left + width / 2)
          .attr("y", height / 2)
          .style("text-anchor", "middle")
          .style("font", "8pt sans-serif")
          .text("Data unavailable");
      return;
    }

    g.append("text")
        .attr("class", "chart-title")
        .attr("x", margin.left + width / 2)
        .attr("y", 0)
        .style("text-anchor", "middle")
        .style("font", "8pt sans-serif")
        .text("Ops/sec over time");

    // Plot data.

    const view = g.append("g")
        .attr("class", "view");

    let values = [];
    for (let key in rawData) {
        values.push({
            id: key,
            values: rawData[key].data,
        });
    }

    const line = d3.line()
        .x(d => x(d.elapsed))
        .y(d => y(d.opsSec));

    const path = view.selectAll(".line1")
        .data(values)
        .enter()
        .append("path")
        .attr("class", "line1")
        .attr("d", d => line(d.values))
        .style("stroke", d => z(d.id));

    // Draw a horizontal line for the calculated ops/sec average.

    view.append("path")
        .attr("d", d3.line()([[x(0), y(opsSec)], [x(maxX), y(opsSec)]]))
        .attr("stroke", "black")
        .attr("stroke-width", "2")
        .style("stroke-dasharray", ("2, 5"));
}
