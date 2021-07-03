use std::fmt::Display;
use std::hash::Hash;

use rand::prelude::SliceRandom;
use rand::SeedableRng;

use crate::engine::field::field::Field;
use crate::utils::dbdashmap::DBDashMap;

#[derive(Clone)]
pub enum EdgeOptions<L: Clone + Hash + Display> {
    Simple,
    Labeled(L),
    Weighted(f64),
    WeightedLabeled(L, f64),
}
//use EdgeOptions::{Simple, Labeled, Weighted, WeightedLabeled};

#[derive(Clone)]
pub struct Edge<O: Hash + Eq + Clone + Display, L: Clone + Hash + Display> {
    pub u: O,
    pub v: O,
    pub label: Option<L>,
    pub weight: Option<f64>,
}

impl<O: Hash + Eq + Clone + Display, L: Clone + Hash + Display> Edge<O, L> {
    pub fn new(u_node: O, v_node: O, edge_options: EdgeOptions<L>) -> Edge<O, L> {
        match edge_options {
            EdgeOptions::Simple => Edge {
                u: u_node,
                v: v_node,
                label: None,
                weight: None,
            },
            EdgeOptions::Labeled(l) => Edge {
                u: u_node,
                v: v_node,
                label: Some(l),
                weight: None,
            },
            EdgeOptions::Weighted(w) => Edge {
                u: u_node,
                v: v_node,
                label: None,
                weight: Some(w),
            },
            EdgeOptions::WeightedLabeled(l, w) => Edge {
                u: u_node,
                v: v_node,
                label: Some(l),
                weight: Some(w),
            },
        }
    }
}

pub struct Network<O: Hash + Eq + Clone + Display, L: Clone + Hash + Display> {
    pub edges: DBDashMap<O, Vec<Edge<O, L>>>,
    pub direct: bool,
}

/**
Generate an undirected network based on
Barabási-Albert’s preferential attachment model
*/
#[macro_export]
macro_rules! preferential_attachment_BA {
    (  $nodes:expr, $network:expr, $node_type:ty, $edge_opt:ty) => {{
        let n_nodes = $nodes.len();
        let mut net: Network<$node_type, $edge_opt> = $network;
        net.remove_all_edges();

        if n_nodes == 0 {
            return;
        }
        net.add_node(&$nodes[0]);
        net.edges.update();
        if n_nodes == 1 {
            return;
        }
        net.add_node(&$nodes[1]);

        net.add_node(&$nodes[0], &$nodes[1], Simple);
        net.edges.update();

        let init_edge: usize = 1;

        for i in 2..n_nodes {
            let node = $nodes[i] as $node_type;

            net.add_prob_edge(&node, &init_edge);
            net.edges.update();
        }

        $network = net;
    }};

    (  $nodes:expr, $network:expr, $node_type:ty, $edge_opt:ty, $init_edges:expr) => {{
        let n_nodes = $nodes.len();
        let edge_to_gen = $init_edges as usize;
        //let mut net: Network<$node_type, $edge_opt> = $network;
        $network.remove_all_edges();

        if n_nodes == 0 {
            return;
        }
        $network.add_node(&$nodes[0]);
        $network.edges.update();
        if n_nodes == 1 {
            return;
        }
        $network.add_node(&$nodes[1]);

        $network.add_edge(&$nodes[0], &$nodes[1], Simple);
        $network.edges.update();
        for i in 2..n_nodes {
            let node = $nodes[i] as $node_type;

            $network.add_prob_edge(&node, &edge_to_gen);
            $network.edges.update();
        }

        //$network = net;
    }};
}

impl<O: Hash + Eq + Clone + Display, L: Clone + Hash + Display> Network<O, L> {
    pub fn new(d: bool) -> Network<O, L> {
        Network {
            edges: DBDashMap::new(),
            direct: d,
        }
    }

    ///part of "preferential attachment" process
    ///in which new network members prefer to make a connection to the more popular existing members.
    pub fn add_prob_edge(&self, u: &O, n_sample: &usize) {
        let net_nodes = self.edges.w_keys();
        let mut dist: Vec<(&O, i32)> = Vec::new();

        for i in 0..net_nodes.len() {
            let n = &net_nodes[i];
            let n_edges = self.get_edges(n).unwrap().len();
            dist.push((n, n_edges as i32));
        }

        let mut rng = rand::thread_rng(); //rand::rngs::StdRng::seed_from_u64(15);
                                          //let chosen = dist.choose_weighted(&mut rng, |dist| dist.1).unwrap().0;
                                          //self.add_edge(u, chosen, EdgeOptions::Simple);

        let amount: usize = if net_nodes.len() < *n_sample {
            net_nodes.len()
        } else {
            *n_sample
        };

        let choices_list = dist
            .choose_multiple_weighted(&mut rng, amount, |dist| dist.1)
            .unwrap()
            .collect::<Vec<_>>();

        for choice in choices_list {
            self.add_edge(u, choice.0, EdgeOptions::Simple);
        }
    }

    pub fn add_node(&self, u: &O) {
        let vec: Vec<Edge<O, L>> = Vec::new();
        self.edges.insert(u.clone(), vec);
    }

    pub fn add_edge(&self, u: &O, v: &O, edge_options: EdgeOptions<L>) -> Option<Edge<O, L>> {
        //println!("add_edge");
        let e = Edge::new(u.clone(), v.clone(), edge_options.clone());
        match self.edges.get_mut(u) {
            Some(mut uedges) => {
                uedges.push(e.clone());
            }
            None => {
                let mut vec = Vec::new();
                vec.push(e.clone());
                self.edges.insert(u.clone(), vec);
            }
        }
        if !self.direct {
            match self.edges.get_mut(v) {
                Some(mut vedges) => {
                    let e = Edge::new(v.clone(), u.clone(), edge_options.clone());
                    vedges.push(e.clone());
                }
                None => {
                    let mut vec = Vec::new();
                    let e = Edge::new(v.clone(), u.clone(), edge_options.clone());
                    vec.push(e.clone());
                    self.edges.insert(v.clone(), vec);
                }
            }
        }
        Some(e)
    }

    pub fn update_edge(&self, u: &O, v: &O, edge_options: EdgeOptions<L>) -> Option<Edge<O, L>> {
        let e = Edge::new(u.clone(), v.clone(), edge_options);
        let ris = match self.edges.get_mut(u) {
            Some(mut uedges) => {
                //TODO search the edge and change it
                uedges.retain(|entry| {
                    !((entry.u == e.u && entry.v == e.v) || (entry.v == e.u && entry.u == e.v))
                });
                uedges.push(e.clone());
                Some(e.clone())
            }
            None => None,
        };

        if !self.direct {
            match self.edges.get_mut(v) {
                Some(mut uedges) => {
                    //TODO search the edge and change it
                    uedges.retain(|entry| {
                        !((entry.u == e.u && entry.v == e.v) || (entry.v == e.u && entry.u == e.v))
                    });
                    uedges.push(e.clone());
                    //TODO
                }
                None => panic!("Error! undirected edge not found"),
            }
        }
        ris
    }

    pub fn get_nodes(&self) -> Vec<&O> {
        self.edges.keys()
    }

    pub fn get_edges(&self, u: &O) -> Option<&Vec<Edge<O, L>>> {
        self.edges.get(&u)
    }

    pub fn get_edge(&self, u: &O, v: &O) -> Option<Edge<O, L>> {
        match self.edges.get(u) {
            Some(uedges) => {
                for e in uedges {
                    if self.direct && e.u == *u && e.v == *v {
                        return Some(e.clone());
                    } else if !self.direct && ((e.u == *u && e.v == *v) || (e.v == *u && e.u == *v))
                    {
                        return Some(e.clone());
                    }
                }
                None
            }
            None => None,
        }
    }

    pub fn remove_edge(&self, u: &O, v: &O) -> Option<Edge<O, L>> {
        //TODO
        let mut u_edges = self.edges.get_mut(u).unwrap();
        let index = match u_edges
            .iter()
            .position(|entry| (entry.u == *u && entry.v == *v) || (entry.u == *v && entry.v == *u))
        {
            Some(i) => i,
            None => return None,
        };

        let u_edge = u_edges.remove(index);
        std::mem::drop(u_edges);

        if self.direct {
            return Some(u_edge.clone());
        } else {
            let mut v_edges = self.edges.get_mut(v).unwrap();
            v_edges.retain(|entry| {
                !((entry.u == *u && entry.v == *v) || (entry.u == *v && entry.v == *u))
            });
            return Some(u_edge.clone());
        }
    }

    pub fn remove_edges(&self, u: &O) -> Option<Vec<Edge<O, L>>> {
        //TODO remove vector for u and all edges for v nodes
        let nodes = self.edges.w_keys();
        let mut ris = vec![];

        for v in nodes.iter() {
            if v != u {
                match self.remove_edge(v, u) {
                    Some(e) => ris.push(e),
                    None => (),
                }
            }
        }

        Some(ris)
    }

    pub fn remove_all_edges(&self) {
        self.edges.clear();
    }
    pub fn remove_node(&self, u: &O) -> bool {
        match self.remove_edges(u) {
            Some(_) => {
                self.edges.remove(u);
                true
            }
            None => false,
        }
    }
}

impl<O: Hash + Eq + Clone + Display, L: Clone + Hash + Display> Field for Network<O, L> {
    fn update(&mut self) {
        self.edges.update();
    }
    fn lazy_update(&mut self) {
        self.edges.update();
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;
    use std::hash::{Hash, Hasher};

    use crate::engine::field::field::Field;
    use crate::engine::field::network::EdgeOptions::Simple;
    use crate::engine::field::network::Network;

    #[derive(Clone, Copy, Debug)]
    struct Node {
        pub id: u128,
        pub value: u64,
    }

    impl Hash for Node {
        fn hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.id.hash(state);
            //    state.write_u128(self.id);
            //    state.finish();
        }
    }

    impl Eq for Node {}

    impl PartialEq for Node {
        fn eq(&self, other: &Node) -> bool {
            self.id == other.id
        }
    }

    impl fmt::Display for Node {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.id)
        }
    }

    #[test]
    fn network() {
        let mut network = Network::<Node, String>::new(false);
        let first_node = Node { id: 1, value: 1 };
        let second_node = Node { id: 2, value: 2 };
        // Two nodes, first and second, with a single edge connecting the two
        network.add_node(&first_node);
        network.add_node(&second_node);
        network.add_edge(&first_node, &second_node, Simple);

        network.update();

        // Was the network double buffer updated correctly?
        assert!(network.get_edge(&first_node, &second_node).is_some());

        // Are we able to fetch all the connected nodes relative to a specific node?
        let neighbors = network.get_edges(&first_node).unwrap();
        assert!(!neighbors.is_empty());
        let fetched_edge = neighbors.first().unwrap();
        assert_eq!(fetched_edge.u.clone(), first_node);
        assert_eq!(fetched_edge.v.clone(), second_node);

        let mut new_first_node = first_node.clone();
        new_first_node.value = 10;
        let mut new_second_node = second_node.clone();
        new_second_node.value = 20;
        network.update_edge(&new_first_node, &new_second_node, Simple);

        network.update();
        network.update();

        let neighbors = network.get_edges(&first_node).unwrap();
        let fetched_edge = neighbors.first().unwrap();
        assert_eq!(fetched_edge.u.value, 10);
        assert_eq!(fetched_edge.v.value, 20);
    }
}
