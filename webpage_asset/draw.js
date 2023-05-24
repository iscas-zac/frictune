//import * as d3 from 'https://d3js.org/d3.v7.min.js';

function draw() {
    var graph = document.getElementById("graph");
    if (graph != null)
    {
        console.log(graph);
        graph.remove();
    }
    var nodes = JSON.parse(document.getElementById("nodes").textContent);
    var links = JSON.parse(document.getElementById("links").textContent);
    var width = 1200;
    var height = 800;
    const forceNode = d3.forceManyBody();
    const forceLink = d3.forceLink(links).id(d => d.id);
    const simulation = d3.forceSimulation(nodes)
        .force("link", forceLink)
        .force("charge", forceNode)
        //.force("center", d3.forceCenter(width / 4, height / 4))
        .force("x", d3.forceX())
        .force("y", d3.forceY())
        .on("tick", ticked);
    //console.log(d3.forceX());
    const svg = d3.select("body").append("svg")
        .attr("id", "graph")
        .attr("width", width)
        .attr("height", height)
        .attr("viewBox", [-width / 2, -height / 2, width, height])
        .attr("style", "max-width: 100%; height: auto; height: intrinsic;");
    const link = svg.append("g")
        .attr("stroke", "black")
        .attr("stroke-opacity", 1)
        .attr("stroke-width", 2)
        .attr("stroke-linecap", "round")
    .selectAll("line")
    .data(links)
    .join("line");
    const node = svg.append("g")
        .attr("fill", "red")
        .attr("stroke", 2)
        .attr("stroke-opacity", 1)
        .attr("stroke-width", 1)
        .selectAll(".node")
            .data(nodes)
            .enter().append("g")
                .attr("class", "node")
                .call(drag(simulation));
    node.append("text")
        .attr("text-anchor", "middle")
        .attr("class", "caption")
        .attr("style", "display: none;")
        .text(function(d, i) { return d.id + "\n" + d.desc; });
    node.append("circle")
        .attr("r", 4);


    d3.selectAll(".node").on("mouseover", function() {
            d3.select(this).select(".caption").attr("style", "display: inherit;");
        }).on("mouseout", function() {
            d3.select(this).select(".caption").attr("style", "display: none;");
        });

    function ticked() {
        link
            .attr("x1", d => d.source.x)
            .attr("y1", d => d.source.y)
            .attr("x2", d => d.target.x)
            .attr("y2", d => d.target.y);

        node
            .selectAll("circle")
                .attr("cx", d => d.x)
                .attr("cy", d => d.y);
        node
            .selectAll("text")
                .attr("fill", "green")
                .attr("x", d => d.x)
                .attr("y", d => d.y - 20);
    }

    function drag(simulation) {    
        function dragstarted(event) {
            if (!event.active) simulation.alphaTarget(0.3).restart();
            event.subject.fx = event.subject.x;
            event.subject.fy = event.subject.y;
        }

        function dragged(event) {
            event.subject.fx = event.x;
            event.subject.fy = event.y;
        }

        function dragended(event) {
            if (!event.active) simulation.alphaTarget(0);
            event.subject.fx = null;
            event.subject.fy = null;
        }

        return d3.drag()
            .on("start", dragstarted)
            .on("drag", dragged)
            .on("end", dragended);
    }
}

draw();
document.getElementById("redraw").onclick = draw;