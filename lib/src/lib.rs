pub mod core;
pub mod dag_creator;
pub mod dag_scheduler;
pub mod dag_set_scheduler;
pub mod fixed_priority_scheduler;
pub mod global_edf_scheduler;
pub mod graph_extension;
pub mod homogeneous;
pub mod log;
pub mod processor;
pub mod util;

#[cfg(any(test, feature = "test-helpers"))]
pub mod tests_helper {
    use petgraph::Graph;

    use crate::graph_extension::NodeData;
    use std::collections::BTreeMap;

    pub fn create_node(id: i32, key: &str, value: i32) -> NodeData {
        let mut params = BTreeMap::new();
        params.insert(key.to_string(), value);
        NodeData { id, params }
    }

    // 2014_ECRTS_federated_original/src/federated.rs
    pub fn create_high_utilization_dag() -> Graph<NodeData, i32> {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = {
            let mut params = BTreeMap::new();
            params.insert("execution_time".to_owned(), 4);
            params.insert("period".to_owned(), 10);
            dag.add_node(NodeData { id: 3, params })
        };
        let n1 = dag.add_node(create_node(1, "execution_time", 4));
        let n2 = dag.add_node(create_node(2, "execution_time", 3));
        let n3 = dag.add_node(create_node(3, "execution_time", 3));
        dag.add_edge(n0, n1, 1);
        dag.add_edge(n0, n2, 1);
        dag.add_edge(n0, n3, 1);

        dag
    }

    pub fn create_low_utilization_dag() -> Graph<NodeData, i32> {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = {
            let mut params = BTreeMap::new();
            params.insert("execution_time".to_owned(), 3);
            params.insert("period".to_owned(), 30);
            dag.add_node(NodeData { id: 2, params })
        };
        let n1 = dag.add_node(create_node(0, "execution_time", 3));
        let n2 = dag.add_node(create_node(1, "execution_time", 4));

        dag.add_edge(n0, n1, 1);
        dag.add_edge(n0, n2, 1);
        dag
    }
}
