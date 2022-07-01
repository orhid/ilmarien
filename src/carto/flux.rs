use crate::carto::{
    brane::{Brane, Resolution},
    datum::DatumZa,
    honeycomb::HoneyCellToroidal,
};
use petgraph::graph::{Graph, NodeIndex};
use std::{collections::HashMap, ops::Sub};

pub struct Flux<T> {
    pub graph: Graph<DatumZa, T>,
    pub roots: Vec<NodeIndex>,
    pub resolution: Resolution,
}

impl<T: Copy + PartialOrd + Sub<Output = T>> From<Brane<T>> for Flux<T> {
    fn from(brane: Brane<T>) -> Self {
        // this places roots at local minima
        //    we could copy this thing with minimal changes to construct
        //    a flux with roots at loacl maxima TODO

        let mut graph = Graph::<DatumZa, T>::new();
        let mut nodes = HashMap::<DatumZa, NodeIndex>::new();
        let mut roots = Vec::<NodeIndex>::new();

        for datum in (0..brane.resolution.square()).map(|j| DatumZa::enravel(j, brane.resolution)) {
            nodes.insert(datum, graph.add_node(datum));
        }

        for jndex in 0..brane.resolution.square() {
            let datum = DatumZa::enravel(jndex, brane.resolution);
            let targets = datum
                .ambit_toroidal(brane.resolution.into())
                .into_iter()
                .filter(|source| brane.grid[source.unravel(brane.resolution)] < brane.grid[jndex])
                .collect::<Vec<DatumZa>>();
            match targets.into_iter().min_by(|a, b| {
                brane.grid[a.unravel(brane.resolution)]
                    .partial_cmp(&brane.grid[b.unravel(brane.resolution)])
                    .unwrap()
            }) {
                Some(target) => {
                    let _ = graph.add_edge(
                        nodes[&datum],
                        nodes[&target],
                        brane.grid[jndex] - brane.grid[target.unravel(brane.resolution)],
                    );
                }
                None => roots.push(nodes[&datum]),
            }
        }

        Self {
            graph,
            roots,
            resolution: brane.resolution,
        }
    }
}
