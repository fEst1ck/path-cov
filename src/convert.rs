//! Converts control flow graphs to regular expressions

use std::fmt::Debug;
use std::sync::Arc;

use crate::extern_cfg::{BlockID, FunID};
use crate::intern_cfg::CFG;
use crate::re::RegExp;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::{EdgeRef, IntoEdges};
use petgraph::Direction::{Incoming, Outgoing};
use petgraph::dot::Dot;

/// Generalized NFA where the transitions are `RegExp<Alphabet, Name>`
#[derive(Debug)]
pub struct GNFA<Alphabet, Name> {
    start_state: NodeIndex,
    accepting_state: NodeIndex,
    pub the_graph: Graph<(), Arc<RegExp<Alphabet, Name>>>,
}

// In our case, Alphabet is the BlockID, Name is the FunID
#[derive(Debug, Clone)]
pub enum Node<Alphabet, Name> {
    Literal(Alphabet),
    Var(Name),
    Extern,
}

impl<Alphabet, Name> Node<Alphabet, Name> {
    pub fn to_re(self) -> RegExp<Alphabet, Name> {
        match self {
            Node::Literal(char) => RegExp::Literal(char),
            Node::Var(var) => RegExp::Var(var),
            Node::Extern => RegExp::Epsilon,
        }
    }
}

impl<Alphabet: Eq + Clone + Ord + Debug, Name: Eq + Clone + Ord + Debug> GNFA<Alphabet, Name> {
    /// Construct a `GNFA` corresponding to cfg `g`.
    ///
    /// The language accepted is the set of execution paths of `g`.
    #[allow(dead_code)]
    pub fn from_cfg<E>(g: Graph<Alphabet, E>, entry: NodeIndex, exit: NodeIndex) -> Self {
        let mut the_graph = g.map(
            |_node_id, _weight| (),
            |edge_id, _weight| {
                let dst_node_id = g.edge_endpoints(edge_id).unwrap().1;
                let dst_node_weight = g.node_weight(dst_node_id).unwrap();
                Arc::new(RegExp::Literal(dst_node_weight.clone()))
            },
        );
        let start_state = the_graph.add_node(());
        the_graph.add_edge(
            start_state,
            entry,
            Arc::new(RegExp::Literal(g.node_weight(entry).unwrap().clone())),
        );
        Self {
            start_state,
            accepting_state: exit,
            the_graph,
        }
    }

    /// Return the number of states of the GNFA.
    pub fn num_states(&self) -> usize {
        self.the_graph.node_count()
    }

    // Return the idx of the next state to rip.
    // Panics if there are no state left to rip.
    fn next_to_rip(&self) -> NodeIndex {
        for v in self.the_graph.node_indices() {
            if v != self.start_state && v != self.accepting_state {
                return v;
            }
        }
        unreachable!()
    }

    // add edge, if the edge already exist, add the weight to it with RegExp:Alter
    fn add_arrow(&mut self, s: NodeIndex, t: NodeIndex, arrow: Arc<RegExp<Alphabet, Name>>) {
        match self.the_graph.find_edge(s, t) {
            Some(e) => {
                self.the_graph[e] = self
                    .the_graph
                    .edge_weight(e)
                    .map(|old_arrow| {
                        let old_arrow_copy = old_arrow.clone();
                        RegExp::alter(old_arrow_copy, arrow)
                    })
                    .unwrap();
            }
            None => {
                self.the_graph.add_edge(s, t, arrow);
            }
        }
    }

    // Rip state s_rip.
    // Panics if s is the start or end state.
    fn rip_state(&mut self, s_rip: NodeIndex) {
        let e_rip = self.the_graph.find_edge(s_rip, s_rip);
        let e_rip_weight = e_rip.map(|e| self.the_graph.edge_weight(e).unwrap().clone());
        let in_edges = self
            .the_graph
            .edges_directed(s_rip, Incoming)
            .map(|x| x.id())
            .collect::<Vec<_>>();
        let out_edges = self
            .the_graph
            .edges_directed(s_rip, Outgoing)
            .map(|x| x.id())
            .collect::<Vec<_>>();
        // add new arrows
        for &in_edge in &in_edges {
            for &out_edge in &out_edges {
                let s_in = self.the_graph.edge_endpoints(in_edge).unwrap().0;
                let s_out = self.the_graph.edge_endpoints(out_edge).unwrap().1;
                if s_in == s_rip || s_out == s_rip {
                    continue;
                }
                let e_in = self.the_graph.edge_weight(in_edge).unwrap().clone();
                let e_out = self.the_graph.edge_weight(out_edge).unwrap().clone();
                let e_new = match e_rip {
                    Some(_e_rip_id) => {
                        // e_in(e_rip)*e_out
                        RegExp::concat(
                            e_in,
                            RegExp::concat(
                                Arc::new(RegExp::star(e_rip_weight.as_ref().unwrap().clone())),
                                e_out,
                            ),
                        )
                    }
                    None => RegExp::concat(e_in, e_out),
                };
                self.add_arrow(s_in, s_out, e_new);
            }
        }
        // remove `s_rip` and associated edges
        assert!(self.the_graph.remove_node(s_rip).is_some());
        // removing `s_rip` may invalidte `start_state` or `accepting_state`
        if self.start_state.index() == self.the_graph.node_count() {
            self.start_state = s_rip;
        }
        if self.accepting_state.index() == self.the_graph.node_count() {
            self.accepting_state = s_rip;
        }
    }

    /// Reduce `self` so that it ends with only 2 states.
    /// The language accepted doesn't change.
    pub fn reduce(&mut self) {
        while self.num_states() > 2 {
        let s_rip = self.next_to_rip();
            self.rip_state(s_rip);
            // println!("after ripping {:?} {:?}", s_rip, Dot::new(&self.the_graph));
        }
    }

    /// Return a reference to an edge from the start state to the accepting state.
    pub fn start_to_end(&self) -> &RegExp<Alphabet, Name> {
        assert!(self.the_graph.edges_connecting(self.start_state, self.accepting_state).count() == 1);
        let idx = self
            .the_graph
            .find_edge(self.start_state, self.accepting_state)
            .unwrap();
        self.the_graph.edge_weight(idx).unwrap()
    }
}

impl GNFA<BlockID, FunID> {
    /// Construct a `GNFA` corresponding to cfg `g`.
    ///
    /// The language accepted is the set of execution paths of `g`.
    pub fn from_intern_cfg(graph: CFG<BlockID, FunID>) -> Self {
        let CFG { entry, exit, graph } = graph;
        let mut the_graph = graph.map(
            |_node_id, _weight| (),
            |edge_id, _weight| {
                let dst_node_id = graph.edge_endpoints(edge_id).unwrap().1;
                let dst_node_weight = graph.node_weight(dst_node_id).unwrap();
                Arc::new(dst_node_weight.clone().to_re())
            },
        );
        let start_state = the_graph.add_node(());
        the_graph.add_edge(
            start_state,
            entry,
            Arc::new(graph.node_weight(entry).unwrap().clone().to_re()),
        );
        let exit_nodes: Vec<_> = the_graph.node_indices().filter(|node_idx| the_graph.neighbors(*node_idx).count() == 0).collect();
        assert!(exit_nodes.len() > 0);
        if exit_nodes.len() > 1 {
            let exit_node = the_graph.add_node(());
            for node in exit_nodes {
                the_graph.add_edge(node, exit_node, Arc::new(RegExp::Epsilon));
            }
            Self {
                start_state,
                accepting_state: exit_node,
                the_graph
            }
        } else {
            Self {
                start_state,
                accepting_state: exit,
                the_graph,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test1() {
        let mut g: Graph<usize, ()> = Graph::new();
        let v1 = g.add_node(1);
        let v4 = g.add_node(4);
        let v2 = g.add_node(2);
        let v3 = g.add_node(3);
        g.add_edge(v1, v2, ());
        g.add_edge(v1, v3, ());
        g.add_edge(v2, v1, ());
        g.add_edge(v2, v4, ());
        g.add_edge(v3, v1, ());
        g.add_edge(v3, v4, ());
        let mut gnfa: GNFA<_, ()> = GNFA::from_cfg(g, v1, v4);
        gnfa.reduce();
        print!("{:?}", gnfa.start_to_end());
    }

    #[test]
    pub fn test2() {
        let mut g: Graph<usize, ()> = Graph::new();
        let v1 = g.add_node(1);
        let v2 = g.add_node(2);
        let v3 = g.add_node(3);
        let v4 = g.add_node(4);
        g.add_edge(v1, v2, ());
        g.add_edge(v2, v3, ());
        g.add_edge(v3, v4, ());
        g.add_edge(v3, v2, ());
        let mut gnfa: GNFA<_, ()> = GNFA::from_cfg(g, v1, v4);
        gnfa.reduce();
        println!("{:?}", gnfa.start_to_end());
    }
}
