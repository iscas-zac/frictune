#[derive(serde::Serialize)]
struct Node {
    id: String,
    desc: String,
}

#[derive(serde::Serialize)]
struct Link {
    source: String,
    target: String,
    strength: f32,
}

#[cfg(target_arch = "wasm32")]
pub fn export_nodes_json(db: &mut frictune::db::crud::Database) -> String {
    let tag_names = frictune::Tag::get_tags(db);
    serde_json::to_string(&tag_names.iter()
        .map(|t| {
            let tag = frictune::Tag::new(t);
            Node { id: t.into(), desc: tag.qd_sync(db).unwrap_or_default() } 
        }).collect::<Vec<_>>()
    ).unwrap()
}

#[cfg(target_arch = "wasm32")]
pub fn export_links_json(db: &mut frictune::db::crud::Database) -> String {
    let tag_names = frictune::Tag::get_tags(db);
    serde_json::to_string(&tag_names.iter()
        .flat_map(|t| {
            let tag = frictune::Tag::new(t);
            tag.qtrd(db).iter().map(|(name, _, weight)| {
                Link { source: t.into(), target: name.into(), strength: weight.unwrap_or_default() }
            }).collect::<Vec<_>>()
        }).collect::<Vec<_>>()
    ).unwrap()
}

#[cfg(target_arch = "wasm32")]
pub fn d3play(db: &mut frictune::db::crud::Database) {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");
    let nodes = document.create_element("p").expect("create nodes failed");
    nodes.set_text_content(Some(&export_nodes_json(db)));
    let links = document.create_element("p").expect("create nodes failed");
    links.set_text_content(Some(&export_links_json(db)));
    js_sys::eval(r##"

    var nodes = [];
    var links = [{ source: "11", target: "22" }];
    var width = 800;
    var height = 600;
    var graph = d3.select("body").append("svg")
        .attr("width", width)
        .attr("height", height);

    const forceNode = d3.forceManyBody();
    const forceLink = d3.forceLink(links).id(d => d.id);
    const simulation = d3.forceSimulation(nodes)
        .force("link", forceLink)
        .force("charge", forceNode)
        .force("center", d3.forceCenter(width / 2, height / 2))
        .on("tick", ticked);
    const svg = d3.select("#viz_area")
        .attr("style", "max-width: 100%; height: auto; height: intrinsic;");
    const link = svg.append("g")
            .attr("stroke", "black")
            .attr("stroke-opacity", 1)
            .attr("stroke-width", 10)
            .attr("stroke-linecap", "round")
        .selectAll("line")
        .data(links)
        .join("line");
    const node = svg.append("g")
            .attr("fill", "red")
            .attr("stroke", 10)
            .attr("stroke-opacity", 1)
            .attr("stroke-width", 1)
            .selectAll(".node")
                .data(nodes)
                .enter().append("g")
                    .attr("class", "node")
                    .call(drag(simulation));
    node.append("text")
            .attr("text-anchor", "middle")
            .text(function(d, i) { return d.tag; });
    node.append("circle")
            .attr("r", 20)
            .call(drag(simulation));
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
    "##);
}