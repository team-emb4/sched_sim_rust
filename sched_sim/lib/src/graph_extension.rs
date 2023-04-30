use petgraph::algo::toposort;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction::{Incoming, Outgoing};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::f32;

use crate::graph_extension_helper::*;

const SOURCE_NODE_ID: i32 = -1;
const SINK_NODE_ID: i32 = -2;

/// custom error type for graph operations
#[derive(Debug)]
pub enum CustomError {
    DuplicateId,
}

/// custom node data structure for dag nodes (petgraph)
#[derive(Debug, Clone)]
pub struct NodeData {
    pub id: i32,
    pub params: HashMap<String, f32>,
}

impl NodeData {
    pub fn new(id: i32, key: String, value: f32) -> NodeData {
        let mut params = HashMap::new();
        params.insert(key, value);
        NodeData { id, params }
    }
}

pub trait GraphExtension {
    fn add_dummy_source_node(&mut self);
    fn add_dummy_sink_node(&mut self);
    fn remove_dummy_source_node(&mut self);
    fn remove_dummy_sink_node(&mut self);
    fn get_critical_paths(&mut self) -> Vec<Vec<NodeIndex>>;
    fn add_node_with_check(&mut self, node_data: NodeData) -> NodeIndex;
}

impl GraphExtension for Graph<NodeData, f32> {
    fn add_dummy_source_node(&mut self) {
        for node_index in self.node_indices() {
            if self[node_index].id == SOURCE_NODE_ID {
                panic!("The dummy source node has already been added.");
            }
        }
        let dummy_node = self.add_node_with_check(NodeData::new(
            SOURCE_NODE_ID,
            "execution_time".to_owned(),
            0.0,
        ));
        let nodes = self
            .node_indices()
            .filter(|&i| self.edges_directed(i, Incoming).next().is_none())
            .collect::<Vec<_>>();

        for node_index in nodes {
            if node_index != dummy_node {
                self.add_edge(dummy_node, node_index, 0.0);
            }
        }
    }
    fn add_dummy_sink_node(&mut self) {
        for node_index in self.node_indices() {
            if self[node_index].id == SINK_NODE_ID {
                panic!("The dummy sink node has already been added.");
            }
        }
        let dummy_node = self.add_node_with_check(NodeData::new(
            SINK_NODE_ID,
            "execution_time".to_owned(),
            0.0,
        ));
        let nodes = self
            .node_indices()
            .filter(|&i| self.edges_directed(i, Outgoing).next().is_none())
            .collect::<Vec<_>>();

        for node_index in nodes {
            if node_index != dummy_node {
                self.add_edge(node_index, dummy_node, 0.0);
            }
        }
    }
    fn remove_dummy_source_node(&mut self) {
        let node_to_remove = self
            .node_indices()
            .find(|&i| self[i].id == SOURCE_NODE_ID)
            .expect("Could not find dummy source node");
        let incoming_edges = self
            .edges_directed(node_to_remove, Incoming)
            .map(|e| e.id())
            .collect::<Vec<_>>();
        for edge_id in incoming_edges {
            self.remove_edge(edge_id);
        }
        self.remove_node(node_to_remove);
    }
    fn remove_dummy_sink_node(&mut self) {
        let node_to_remove = self
            .node_indices()
            .find(|&i| self[i].id == SINK_NODE_ID)
            .expect("Could not find dummy sink node");
        let outgoing_edges = self
            .edges_directed(node_to_remove, Outgoing)
            .map(|e| e.id())
            .collect::<Vec<_>>();
        for edge_id in outgoing_edges {
            self.remove_edge(edge_id);
        }
        self.remove_node(node_to_remove);
    }

    /// Returns the critical path of a DAG
    /// Multiple critical paths are obtained using Breadth-First Search, BFS
    ///
    /// # Arguments
    ///
    /// * `dag` - dag object. each node contains execution time information.
    ///
    /// # Returns
    ///
    /// * `critical path` -containing the nodes in the critical path. Multiple critical paths may exist. so the return value is a vector of vectors.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph::Graph;
    /// use std::collections::HashMap;
    /// use lib::graph_extension::NodeData;
    /// use lib::graph_extension::GraphExtension;
    ///
    /// let mut dag = Graph::<NodeData, f32>::new();
    /// let mut params = HashMap::new();
    /// params.insert("execution_time".to_string(), 1.0);
    /// let n0 = dag.add_node_with_check(NodeData { id: 0, params: params.clone() });
    /// let n1 = dag.add_node_with_check(NodeData { id: 1, params: params.clone() });
    /// dag.add_edge(n0, n1, 1.0);
    /// let critical_path = dag.get_critical_paths();
    /// println!("The critical path is: {:?}", critical_path);
    /// ```
    fn get_critical_paths(&mut self) -> Vec<Vec<NodeIndex>> {
        self.add_dummy_source_node();
        self.add_dummy_sink_node();
        println!("DAG: {:?}", self);
        let earliest_start_times = calculate_earliest_start_times(self);
        let latest_start_times = calculate_latest_start_times(self);
        let sorted_nodes = toposort(&*self, None).unwrap();
        let start_node = sorted_nodes[0];
        let mut critical_paths = Vec::new();
        let mut path_search_queue = VecDeque::new();
        path_search_queue.push_back((start_node, vec![start_node]));

        while let Some((node, mut critical_path)) = path_search_queue.pop_front() {
            let outgoing_edges = self.edges_directed(node, Outgoing);

            if outgoing_edges.clone().count() == 0 {
                critical_path.pop(); // Remove the dummy sink node
                critical_path.remove(0); // Remove the dummy source node
                critical_paths.push(critical_path);
            } else {
                for edge in outgoing_edges {
                    let target_node = edge.target();
                    if earliest_start_times[target_node.index()]
                        == latest_start_times[target_node.index()]
                    {
                        let mut current_critical_path = critical_path.clone();
                        current_critical_path.push(target_node);
                        path_search_queue.push_back((target_node, current_critical_path));
                    }
                }
            }
        }

        critical_paths
    }

    /// check if the graph contains a node with the given id
    fn add_node_with_check(&mut self, node_data: NodeData) -> NodeIndex {
        for node_index in self.node_indices() {
            let existing_node = self.node_weight(node_index).unwrap();
            if existing_node.id == node_data.id {
                panic!("Duplicate id found: {}", node_data.id);
            }
        }
        self.add_node(node_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_critical_paths_multiple() {
        fn create_node(id: i32, key: &str, value: f32) -> NodeData {
            let mut params = HashMap::new();
            params.insert(key.to_string(), value);
            NodeData { id, params }
        }
        let mut dag = Graph::<NodeData, f32>::new();
        let n0 = dag.add_node_with_check(create_node(0, "execution_time", 3.0));
        let n1 = dag.add_node_with_check(create_node(1, "execution_time", 6.0));
        let n2 = dag.add_node_with_check(create_node(2, "execution_time", 45.0));
        let n3 = dag.add_node_with_check(create_node(3, "execution_time", 26.0));
        let n4 = dag.add_node_with_check(create_node(4, "execution_time", 44.0));
        let n5 = dag.add_node_with_check(create_node(5, "execution_time", 26.0));
        let n6 = dag.add_node_with_check(create_node(6, "execution_time", 26.0));
        let n7 = dag.add_node_with_check(create_node(7, "execution_time", 27.0));
        let n8 = dag.add_node_with_check(create_node(8, "execution_time", 43.0));
        dag.add_edge(n0, n1, 1.0);
        dag.add_edge(n1, n2, 1.0);
        dag.add_edge(n1, n3, 1.0);
        dag.add_edge(n1, n4, 1.0);
        dag.add_edge(n2, n5, 1.0);
        dag.add_edge(n3, n6, 1.0);
        dag.add_edge(n4, n7, 1.0);
        dag.add_edge(n5, n8, 1.0);
        dag.add_edge(n6, n8, 1.0);
        dag.add_edge(n7, n8, 1.0);

        let critical_path = dag.get_critical_paths();
        assert_eq!(critical_path.len(), 2);

        assert_eq!(
            critical_path[0]
                .iter()
                .map(|node_index| node_index.index())
                .collect::<Vec<_>>(),
            vec![0_usize, 1_usize, 4_usize, 7_usize, 8_usize]
        );
        assert_eq!(
            critical_path[1]
                .iter()
                .map(|node_index| node_index.index())
                .collect::<Vec<_>>(),
            vec![0_usize, 1_usize, 2_usize, 5_usize, 8_usize]
        );
    }
}