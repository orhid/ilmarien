use crate::carto::{brane::Brane, datum::DatumZa};
use num_traits::identities::Zero;
use ord_subset::{OrdSubset, OrdSubsetIterExt};
use petgraph::graph::{Graph, NodeIndex};
use std::{collections::HashMap, ops::Sub};

/* # fluxes */

pub struct Flux<T> {
    pub graph: Graph<DatumZa, T>,
    pub roots: Vec<NodeIndex>,
    pub resolution: usize,
    pub variable: String,
}

impl<T: Copy + OrdSubset + Sub<Output = T> + Zero> From<&Brane<T>> for Flux<T> {
    fn from(brane: &Brane<T>) -> Self {
        let mut graph = Graph::<DatumZa, T>::new();
        let mut nodes = HashMap::<DatumZa, NodeIndex>::new();
        let mut roots = Vec::<NodeIndex>::new();
        for datum in brane.iter_exact() {
            let here = graph.add_node(datum);
            nodes.insert(datum, here);
        }
        for datum in brane.iter_exact() {
            let minbr = *brane
                .ambit_exact(&datum)
                .iter()
                .ord_subset_min_by_key(|nbr| brane.read(&nbr))
                .unwrap();
            let dif = brane.read(&datum) - brane.read(&minbr);
            if dif > T::zero() {
                graph.add_edge(nodes[&datum], nodes[&minbr], dif);
            } else {
                roots.push(nodes[&datum]);
            }
        }
        Self {
            graph,
            roots,
            resolution: brane.resolution,
            variable: brane.variable.clone(),
        }
    }
}
