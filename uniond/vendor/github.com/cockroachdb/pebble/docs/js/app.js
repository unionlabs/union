// TODO(peter)
// - Save pan/zoom settings in query params
//
// TODO(travers): There exists an awkward ordering script loading issue where
// write-throughput.js is loaded first, but contains references to functions
// defined in this file. Work out a better way of modularizing this code.

const parseTime = d3.timeParse("%Y%m%d");
const formatTime = d3.timeFormat("%b %d");
const dateBisector = d3.bisector(d => d.date).left;

let minDate;
let max = {
    date: new Date(),
    perChart: {},
    opsSec: 0,
    readBytes: 0,
    writeBytes: 0,
    readAmp: 0,
    writeAmp: 0
};
let usePerChartMax = false;
let detail;
let detailName;
let detailFormat;

let annotations = [];

function getMaxes(chartKey) {
    return usePerChartMax ? max.perChart[chartKey] : max;
}

function styleWidth(e) {
    const width = +e.style("width").slice(0, -2);
    return Math.round(Number(width));
}

function styleHeight(e) {
    const height = +e.style("height").slice(0, -2);
    return Math.round(Number(height));
}

function pathGetY(path, x) {
    // Walk along the path using binary search to locate the point
    // with the supplied x value.
    let start = 0;
    let end = path.getTotalLength();
    while (start < end) {
        const target = (start + end) / 2;
        const pos = path.getPointAtLength(target);
        if (Math.abs(pos.x - x) < 0.01) {
            // Close enough.
            return pos.y;
        } else if (pos.x > x) {
            end = target;
        } else {
            start = target;
        }
    }
    return path.getPointAtLength(start).y;
}

// Pretty formatting of a number in human readable units.
function humanize(s) {
    const iecSuffixes = [" B", " KB", " MB", " GB", " TB", " PB", " EB"];
    if (s < 10) {
        return "" + s;
    }
    let e = Math.floor(Math.log(s) / Math.log(1024));
    let suffix = iecSuffixes[Math.floor(e)];
    let val = Math.floor(s / Math.pow(1024, e) * 10 + 0.5) / 10;
    return val.toFixed(val < 10 ? 1 : 0) + suffix;
}

function dirname(path) {
    return path.match(/.*\//)[0];
}

function equalDay(d1, d2) {
    return (
        d1.getYear() == d2.getYear() &&
        d1.getMonth() == d2.getMonth() &&
        d1.getDate() == d2.getDate()
    );
}

function computeSegments(data) {
    return data.reduce(function(segments, d) {
        if (segments.length == 0) {
            segments.push([d]);
            return segments;
        }

        const lastSegment = segments[segments.length - 1];
        const lastDatum = lastSegment[lastSegment.length - 1];
        const days = Math.round(
            (d.date.getTime() - lastDatum.date.getTime()) /
                (24 * 60 * 60 * 1000)
        );
        if (days == 1) {
            lastSegment.push(d);
        } else {
            segments.push([d]);
        }
        return segments;
    }, []);
}

function computeGaps(segments) {
    let gaps = [];
    for (let i = 1; i < segments.length; ++i) {
        const last = segments[i - 1];
        const cur = segments[i];
        gaps.push([last[last.length - 1], cur[0]]);
    }

    // If the last day is not equal to the current day, add a gap that
    // spans to the current day.
    const last = segments[segments.length - 1];
    const lastDay = last[last.length - 1];
    if (!equalDay(lastDay.date, max.date)) {
        const maxDay = Object.assign({}, lastDay);
        maxDay.date = max.date;
        gaps.push([lastDay, maxDay]);
    }
    return gaps;
}

function renderChart(chart) {
    const chartKey = chart.attr("data-key");
    const vals = data[chartKey];

    const svg = chart.html("");

    const margin = { top: 25, right: 60, bottom: 25, left: 60 };

    const width = styleWidth(svg) - margin.left - margin.right,
        height = styleHeight(svg) - margin.top - margin.bottom;

    const defs = svg.append("defs");
    const filter = defs
        .append("filter")
        .attr("id", "textBackground")
        .attr("x", 0)
        .attr("y", 0)
        .attr("width", 1)
        .attr("height", 1);
    filter.append("feFlood").attr("flood-color", "white");
    filter.append("feComposite").attr("in", "SourceGraphic");

    defs
        .append("clipPath")
        .attr("id", chartKey)
        .append("rect")
        .attr("x", 0)
        .attr("y", -margin.top)
        .attr("width", width)
        .attr("height", margin.top + height + 10);

    const title = svg
        .append("text")
        .attr("class", "chart-title")
        .attr("x", margin.left + width / 2)
        .attr("y", 15)
        .style("text-anchor", "middle")
        .style("font", "8pt sans-serif")
        .text(chartKey);

    const g = svg
        .append("g")
        .attr("transform", "translate(" + margin.left + "," + margin.top + ")");

    const x = d3.scaleTime().range([0, width]);
    const x2 = d3.scaleTime().range([0, width]);
    const y1 = d3.scaleLinear().range([height, 0]);
    const z = d3.scaleOrdinal(d3.schemeCategory10);
    const xFormat = formatTime;

    x.domain([minDate, max.date]);
    x2.domain([minDate, max.date]);

    y1.domain([0, getMaxes(chartKey).opsSec]);

    const xAxis = d3.axisBottom(x).ticks(5);

    g
        .append("g")
        .attr("class", "axis axis--x")
        .attr("transform", "translate(0," + height + ")")
        .call(xAxis);
    g
        .append("g")
        .attr("class", "axis axis--y")
        .call(d3.axisLeft(y1).ticks(5));

    if (!vals) {
        // That's all we can draw for an empty chart.
        svg
            .append("text")
            .attr("x", margin.left + width / 2)
            .attr("y", margin.top + height / 2)
            .style("text-anchor", "middle")
            .style("font", "8pt sans-serif")
            .text("No data");
        return;
    }

    const view = g
        .append("g")
        .attr("class", "view")
        .attr("clip-path", "url(#" + chartKey + ")");

    const triangle = d3
        .symbol()
        .type(d3.symbolTriangle)
        .size(12);
    view
        .selectAll("path.annotation")
        .data(annotations)
        .enter()
        .append("path")
        .attr("class", "annotation")
        .attr("d", triangle)
        .attr("stroke", "#2b2")
        .attr("fill", "#2b2")
        .attr(
            "transform",
            d => "translate(" + (x(d.date) + "," + (height + 5) + ")")
        );

    view
        .selectAll("line.annotation")
        .data(annotations)
        .enter()
        .append("line")
        .attr("class", "annotation")
        .attr("fill", "none")
        .attr("stroke", "#2b2")
        .attr("stroke-width", "1px")
        .attr("stroke-dasharray", "1 2")
        .attr("x1", d => x(d.date))
        .attr("x2", d => x(d.date))
        .attr("y1", 0)
        .attr("y2", height);

    // Divide the data into contiguous days so that we can avoid
    // interpolating days where there is missing data.
    const segments = computeSegments(vals);
    const gaps = computeGaps(segments);

    const line1 = d3
        .line()
        .x(d => x(d.date))
        .y(d => y1(d.opsSec));
    const path1 = view
        .selectAll(".line1")
        .data(segments)
        .enter()
        .append("path")
        .attr("class", "line1")
        .attr("d", line1)
        .style("stroke", d => z(0));

    view
        .selectAll(".line1-gaps")
        .data(gaps)
        .enter()
        .append("path")
        .attr("class", "line1-gaps")
        .attr("d", line1)
        .attr("opacity", 0.8)
        .style("stroke", d => z(0))
        .style("stroke-dasharray", "1 2");

    let y2 = d3.scaleLinear().range([height, 0]);
    let line2;
    let path2;
    if (detail) {
        y2 = d3.scaleLinear().range([height, 0]);
        y2.domain([0, detail(getMaxes(chartKey))]);
        g
            .append("g")
            .attr("class", "axis axis--y")
            .attr("transform", "translate(" + width + ",0)")
            .call(
                d3
                    .axisRight(y2)
                    .ticks(5)
                    .tickFormat(detailFormat)
            );

        line2 = d3
            .line()
            .x(d => x(d.date))
            .y(d => y2(detail(d)));
        path2 = view
            .selectAll(".line2")
            .data(segments)
            .enter()
            .append("path")
            .attr("class", "line2")
            .attr("d", line2)
            .style("stroke", d => z(1));
        view
            .selectAll(".line2-gaps")
            .data(gaps)
            .enter()
            .append("path")
            .attr("class", "line2-gaps")
            .attr("d", line2)
            .attr("opacity", 0.8)
            .style("stroke", d => z(1))
            .style("stroke-dasharray", "1 2");
    }

    const updateZoom = function(t) {
        x.domain(t.rescaleX(x2).domain());
        g.select(".axis--x").call(xAxis);
        g.selectAll(".line1").attr("d", line1);
        g.selectAll(".line1-gaps").attr("d", line1);
        if (detail) {
            g.selectAll(".line2").attr("d", line2);
            g.selectAll(".line2-gaps").attr("d", line2);
        }
        g
            .selectAll("path.annotation")
            .attr(
                "transform",
                d => "translate(" + (x(d.date) + "," + (height + 5) + ")")
            );
        g
            .selectAll("line.annotation")
            .attr("x1", d => x(d.date))
            .attr("x2", d => x(d.date));
    };
    svg.node().updateZoom = updateZoom;

    const hoverSeries = function(mouse) {
        if (!detail) {
            return 1;
        }
        const mousex = mouse[0];
        const mousey = mouse[1] - margin.top;
        const path1Y = pathGetY(path1.node(), mousex);
        const path2Y = pathGetY(path2.node(), mousex);
        return Math.abs(mousey - path1Y) < Math.abs(mousey - path2Y) ? 1 : 2;
    };

    // This is a bit funky: initDate() initializes the date range to
    // [today-90,today]. We then allow zooming out by 4x which will
    // give a maximum range of 360 days. We limit translation to the
    // 360 day period. The funkiness is that it would be more natural
    // to start at the maximum zoomed amount and then initialize the
    // zoom. But that doesn't work because we want to maintain the
    // existing zoom settings whenever we have to (re-)render().
    const zoom = d3
        .zoom()
        .scaleExtent([0.25, 2])
        .translateExtent([[-width * 3, 0], [width, 1]])
        .extent([[0, 0], [width, 1]])
        .on("zoom", function() {
            const t = d3.event.transform;
            if (!d3.event.sourceEvent) {
                updateZoom(t);
                return;
            }

            d3.selectAll(".chart").each(function() {
                if (this.updateZoom != null) {
                    this.updateZoom(t);
                }
            });

            d3.selectAll(".chart").each(function() {
                this.__zoom = t.translate(0, 0);
            });

            const mouse = d3.mouse(this);
            if (mouse) {
                mouse[0] -= margin.left; // adjust for rect.mouse position
                const date = x.invert(mouse[0]);
                const hover = hoverSeries(mouse);
                d3.selectAll(".chart.ycsb").each(function() {
                    this.updateMouse(mouse, date, hover);
                });
            }
        });

    svg.call(zoom);
    svg.call(zoom.transform, d3.zoomTransform(svg.node()));

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

    svg.node().updateMouse = function(mouse, date, hover) {
        const mousex = mouse[0];
        const mousey = mouse[1];
        const i = dateBisector(vals, date, 1);
        const v =
            i == vals.length
                ? vals[i - 1]
                : mousex - x(vals[i - 1].date) < x(vals[i].date) - mousex
                    ? vals[i - 1]
                    : vals[i];
        const noData = mousex < x(vals[0].date);

        let lineY = height;
        if (!noData) {
            if (hover == 1) {
                lineY = pathGetY(path1.node(), mousex);
            } else {
                lineY = pathGetY(path2.node(), mousex);
            }
        }

        let val, valY, valFormat;
        if (hover == 1) {
            val = v.opsSec;
            valY = y1(val);
            valFormat = d3.format(",.0f");
        } else {
            val = detail(v);
            valY = y2(val);
            valFormat = detailFormat;
        }

        lineHover
            .attr("x1", mousex)
            .attr("x2", mousex)
            .attr("y1", lineY)
            .attr("y2", height);
        marker.attr("transform", "translate(" + x(v.date) + "," + valY + ")");
        dateHover
            .attr("transform", "translate(" + mousex + "," + (height + 8) + ")")
            .text(xFormat(date));
        opsHover
            .attr(
                "transform",
                "translate(" + x(v.date) + "," + (valY - 7) + ")"
            )
            .text(valFormat(val));
    };

    const rect = svg
        .append("rect")
        .attr("class", "mouse")
        .attr("cursor", "move")
        .attr("fill", "none")
        .attr("pointer-events", "all")
        .attr("width", width)
        .attr("height", height + margin.top + margin.bottom)
        .attr("transform", "translate(" + margin.left + "," + 0 + ")")
        .on("mousemove", function() {
            const mouse = d3.mouse(this);
            const date = x.invert(mouse[0]);
            const hover = hoverSeries(mouse);

            let resetTitle = true;
            for (let i in annotations) {
                if (Math.abs(mouse[0] - x(annotations[i].date)) <= 5) {
                    title
                        .style("font-size", "9pt")
                        .text(annotations[i].message);
                    resetTitle = false;
                    break;
                }
            }
            if (resetTitle) {
                title.style("font-size", "8pt").text(chartKey);
            }

            d3.selectAll(".chart").each(function() {
                if (this.updateMouse != null) {
                    this.updateMouse(mouse, date, hover);
                }
            });
        })
        .on("mouseover", function() {
            d3
                .selectAll(".chart")
                .selectAll(".hover")
                .style("opacity", 1.0);
        })
        .on("mouseout", function() {
            d3
                .selectAll(".chart")
                .selectAll(".hover")
                .style("opacity", 0);
        });
}

function renderYCSB() {
    d3.selectAll(".chart.ycsb").each(function(d, i) {
        renderChart(d3.select(this));
    });
}

function initData() {
    for (key in data) {
        data[key] = d3.csvParseRows(data[key], function(d, i) {
            return {
                date: parseTime(d[0]),
                opsSec: +d[1],
                readBytes: +d[2],
                writeBytes: +d[3],
                readAmp: +d[4],
                writeAmp: +d[5]
            };
        });

        const vals = data[key];
        max.perChart[key] = {
            opsSec: d3.max(vals, d => d.opsSec),
            readBytes: d3.max(vals, d => d.readBytes),
            writeBytes: d3.max(vals, d => d.writeBytes),
            readAmp: d3.max(vals, d => d.readAmp),
            writeAmp: d3.max(vals, d => d.writeAmp),
        }
        max.opsSec = Math.max(max.opsSec, max.perChart[key].opsSec);
        max.readBytes = Math.max(max.readBytes, max.perChart[key].readBytes);
        max.writeBytes = Math.max(
            max.writeBytes,
            max.perChart[key].writeBytes,
        );
        max.readAmp = Math.max(max.readAmp, max.perChart[key].readAmp);
        max.writeAmp = Math.max(max.writeAmp, max.perChart[key].writeAmp);
    }

    // Load the write-throughput data and merge with the existing data. We
    // return a promise here to allow us to continue to make progress elsewhere.
    return fetch(writeThroughputSummaryURL())
      .then(response => response.json())
      .then(wtData => {
            for (let key in wtData) {
                data[key] = wtData[key];
            }
      });
}

function initDateRange() {
    max.date.setHours(0, 0, 0, 0);
    minDate = new Date(new Date().setDate(max.date.getDate() - 90));
}

function initAnnotations() {
    d3.selectAll(".annotation").each(function() {
        const annotation = d3.select(this);
        const date = parseTime(annotation.attr("data-date"));
        annotations.push({ date: date, message: annotation.text() });
    });
}

function setQueryParams() {
    var params = new URLSearchParams();
    if (detailName) {
        params.set("detail", detailName);
    }
    if (usePerChartMax) {
        params.set("max", "local");
    }
    var search = "?" + params;
    if (window.location.search != search) {
        window.history.pushState(null, null, search);
    }
}

function setDetail(name) {
    detail = undefined;
    detailFormat = undefined;
    detailName = name;

    switch (detailName) {
        case "readBytes":
            detail = d => d.readBytes;
            detailFormat = humanize;
            break;
        case "writeBytes":
            detail = d => d.writeBytes;
            detailFormat = humanize;
            break;
        case "readAmp":
            detail = d => d.readAmp;
            detailFormat = d3.format(",.1f");
            break;
        case "writeAmp":
            detail = d => d.writeAmp;
            detailFormat = d3.format(",.1f");
            break;
    }

    d3.selectAll(".toggle").classed("selected", false);
    d3.select("#" + detailName).classed("selected", detail != null);
}

function initQueryParams() {
    var params = new URLSearchParams(window.location.search.substring(1));
    setDetail(params.get("detail"));
    usePerChartMax = params.get("max") == "local";
    d3.select("#localMax").classed("selected", usePerChartMax);
}

function toggleDetail(name) {
    const link = d3.select("#" + name);
    const selected = !link.classed("selected");
    link.classed("selected", selected);
    if (selected) {
        setDetail(name);
    } else {
        setDetail(null);
    }
    setQueryParams();
    renderYCSB();
}

function toggleLocalMax() {
    const link = d3.select("#localMax");
    const selected = !link.classed("selected");
    link.classed("selected", selected);
    usePerChartMax = selected;
    setQueryParams();
    renderYCSB();
}

window.onload = function init() {
    d3.selectAll(".toggle").each(function() {
        const link = d3.select(this);
        link.attr("href", 'javascript:toggleDetail("' + link.attr("id") + '")');
    });
    d3.selectAll("#localMax").each(function() {
        const link = d3.select(this);
        link.attr("href", 'javascript:toggleLocalMax()');
    });

    initData().then(_ => {
        initDateRange();
        initAnnotations();
        initQueryParams();

        renderYCSB();
        renderWriteThroughputSummary(data);

        // Use the max date to bisect into the workload data to pluck out the
        // correct datapoint.
        let workloadData = data[writeThroughputWorkload];
        bisectAndRenderWriteThroughputDetail(workloadData, max.date);

        let lastUpdate;
        for (let key in data) {
            const max = d3.max(data[key], d => d.date);
            if (!lastUpdate || lastUpdate < max) {
                lastUpdate = max;
            }
        }
        d3.selectAll(".updated")
            .text("Last updated: " + d3.timeFormat("%b %e, %Y")(lastUpdate));
    })

    // By default, display each panel with its local max, which makes spotting
    // regressions simpler.
    toggleLocalMax();
};

window.onpopstate = function() {
    initQueryParams();
    renderYCSB();
};

window.addEventListener("resize", renderYCSB);
