//! Generate a petgraph DAG object from a yaml file

use log::warn;
use petgraph::graph::Graph;
use petgraph::prelude::*;
use std::collections::HashMap;
use std::fs;
use yaml_rust::Yaml;
use yaml_rust::YamlLoader;

use std::path::PathBuf;

use crate::graph_extension::NodeData;

fn load_yaml(file_path: &str) -> Vec<yaml_rust::Yaml> {
    if !file_path.ends_with(".yaml") && !file_path.ends_with(".yml") {
        panic!("Invalid file type: {}", file_path);
    }
    let file_content = fs::read_to_string(file_path).unwrap();
    YamlLoader::load_from_str(&file_content).unwrap()
}

fn get_minimum_decimal_places(yaml: &Yaml) -> usize {
    let mut minimum_decimal_places = 0;
    match yaml {
        Yaml::Real(real) => {
            let decimal_places = real
                .split('.')
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .chars()
                .count();
            if decimal_places > minimum_decimal_places {
                minimum_decimal_places = decimal_places;
            }
        }
        Yaml::Array(array) => {
            for element in array {
                let decimal_places = get_minimum_decimal_places(element);
                if decimal_places > minimum_decimal_places {
                    minimum_decimal_places = decimal_places;
                }
            }
        }
        Yaml::Hash(hash) => {
            for (_key, value) in hash {
                let decimal_places = get_minimum_decimal_places(value);
                if decimal_places > minimum_decimal_places {
                    minimum_decimal_places = decimal_places;
                }
            }
        }
        _ => {}
    }
    minimum_decimal_places
}

/// load yaml file and return a dag object (petgraph)
///
/// # Arguments
///
/// *  `file_path` - yaml file path
///
/// # Returns
///
/// *  `dag` - dag object (petgraph)
///
/// # Example
///
/// ```
/// use lib::dag_creator::create_dag_from_yaml;
///
/// let dag = create_dag_from_yaml("tests/sample_dags/chain_base_format.yaml");
/// let first_node = dag.node_indices().next().unwrap();
/// let first_edge = dag.edge_indices().next().unwrap();
///
/// let node_num = dag.node_count();
/// let edge_num = dag.edge_count();
/// let node_id = dag[first_node].id;
/// let edge_weight = dag[first_edge];
/// ```
pub fn create_dag_from_yaml(file_path: &str) -> Graph<NodeData, i32> {
    let yaml_docs = load_yaml(file_path);
    let yaml_doc = &yaml_docs[0];
    let int_conversion_factor =
        10f32.powi(get_minimum_decimal_places(yaml_doc).try_into().unwrap()) as i32;
    if int_conversion_factor > 100000 {
        warn!("The number of decimal places is too large. Please reduce the number of decimal places to 5 or less.");
    }

    // Check if nodes and links fields exist
    if let (Some(nodes), Some(links)) = (yaml_doc["nodes"].as_vec(), yaml_doc["links"].as_vec()) {
        let mut dag = Graph::<NodeData, i32>::new();

        // add nodes to dag
        for node in nodes {
            let mut params = HashMap::new();
            let id = node["id"].as_i64().unwrap() as i32;

            // add node parameters to HashMap
            for (key, value) in node.as_hash().unwrap() {
                let key_str = key.as_str().unwrap();
                if key_str != "id" {
                    match value {
                        Yaml::Integer(_i) => {
                            params.insert(
                                key_str.to_owned(),
                                (value.as_i64().unwrap() * int_conversion_factor as i64) as i32,
                            );
                        }
                        Yaml::Real(_r) => {
                            params.insert(
                                key_str.to_owned(),
                                (value.as_f64().unwrap() * int_conversion_factor as f64) as i32,
                            );
                        }
                        _ => {
                            panic!("Unknown type: {}", std::any::type_name::<Yaml>());
                        }
                    }
                }
            }
            dag.add_node(NodeData { id, params });
        }

        // add edges to dag
        for link in links {
            let source = link["source"].as_i64().unwrap() as usize;
            let target = link["target"].as_i64().unwrap() as usize;
            let mut communication_time = 0;

            match &link["communication_time"] {
                Yaml::Integer(communication_time_value) => {
                    communication_time = *communication_time_value as i32 * int_conversion_factor;
                }
                Yaml::Real(communication_time_value) => {
                    communication_time = (communication_time_value.parse::<f32>().unwrap()
                        * int_conversion_factor as f32)
                        as i32;
                }
                Yaml::BadValue => {}
                _ => unreachable!(),
            }
            dag.add_edge(
                NodeIndex::new(source),
                NodeIndex::new(target),
                communication_time,
            );
        }
        dag
    } else {
        panic!("YAML files are not DAG structures.");
    }
}

fn get_yaml_paths_from_dir(dir_path: &str) -> Vec<String> {
    if !std::fs::metadata(dir_path).unwrap().is_dir() {
        panic!("Not a directory");
    }
    let mut file_path_list = Vec::new();
    for dir_entry_result in PathBuf::from(dir_path).read_dir().unwrap() {
        let path = dir_entry_result.unwrap().path();
        let extension = path.extension().unwrap();
        if extension == "yaml" || extension == "yml" {
            file_path_list.push(path.to_str().unwrap().to_string());
        }
    }
    if file_path_list.is_empty() {
        panic!("No YAML file found in {}", dir_path);
    }
    file_path_list
}

/// load yaml files and return a dag_set (dag list)
///
/// # Arguments
///
/// *  `dir_path` - dir path for yaml files
///
/// # Returns
///
/// *  `dag_set` - dag list (petgraph vector)
///
/// # Example
///
/// ```
/// use lib::dag_creator::create_dag_set_from_dir;
/// let dag_set = create_dag_set_from_dir("tests/sample_dags/multiple_yaml_files");
/// let first_node_num = dag_set[0].node_count();
/// let first_edge_num = dag_set[0].edge_count();
/// let first_node_exe_time = dag_set[0][dag_set[0].node_indices().next().unwrap()].params["execution_time"];
/// ```
pub fn create_dag_set_from_dir(dir_path: &str) -> Vec<Graph<NodeData, i32>> {
    let file_path_list = get_yaml_paths_from_dir(dir_path);
    let mut dag_set: Vec<Graph<NodeData, i32>> = Vec::new();

    for file_path in file_path_list {
        let dag = create_dag_from_yaml(&file_path);
        dag_set.push(dag);
    }
    dag_set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_minimum_decimal_places_normal() {
        let yaml_docs = load_yaml("tests/sample_dags/float_params.yaml");
        let yaml_doc = &yaml_docs[0];
        let number_of_digits = get_minimum_decimal_places(yaml_doc);
        assert_eq!(number_of_digits, 1, "number of digits is expected to be 1");
    }
    #[test]
    fn test_create_dag_set_from_dir_multiple_yaml_files() {
        let dag_set = create_dag_set_from_dir("tests/sample_dags/multiple_yaml_files");
        assert_eq!(dag_set.len(), 2, "number of dag_set is expected to be 2");
    }

    #[test]
    fn test_create_dag_set_from_dir_mixing_dif_ext() {
        let dag_set = create_dag_set_from_dir("tests/sample_dags/mixing_different_extensions");
        assert_eq!(dag_set.len(), 1, "number of dag_set is expected to be 1");
    }

    #[test]
    #[should_panic]
    fn test_create_dag_set_from_dir_mixing_not_dag_yaml() {
        create_dag_set_from_dir("tests/sample_dags/mixing_not_dag_yaml");
    }

    #[test]
    #[should_panic]
    fn test_create_dag_set_from_dir_no_yaml_files() {
        create_dag_set_from_dir("tests/sample_dags/no_yaml_files");
    }

    #[test]
    #[should_panic]
    fn test_create_dag_set_from_dir_no_dir() {
        create_dag_set_from_dir("tests/sample_dags/gnp_format.yaml");
    }

    #[test]
    fn test_create_dag_from_yaml_chain_base() {
        let dag = create_dag_from_yaml("tests/sample_dags/chain_base_format.yaml");
        let first_node = dag.node_indices().next().unwrap();
        let last_node = dag.node_indices().last().unwrap();
        let first_edge = dag.edge_indices().next().unwrap();
        let last_edge = dag.edge_indices().last().unwrap();

        assert_eq!(dag.node_count(), 22, "number of nodes is expected to be 22");
        assert_eq!(
            dag[first_node].params.get("execution_time").unwrap(),
            &73,
            "first node execution time is expected to be 73"
        );
        assert_eq!(
            dag[last_node].params.get("execution_time").unwrap(),
            &2,
            "last node execution time is expected to be 2"
        );
        assert_eq!(dag[first_node].id, 0, "first node id is expected to be 0");
        assert_eq!(dag[last_node].id, 21, "last node id is expected to be 21");
        assert_eq!(
            dag[first_node].params.get("period").unwrap(),
            &50,
            "first node period is expected to be 50"
        );
        assert_eq!(dag.edge_count(), 25, "number of edges is expected to be 25");
        assert_eq!(
            dag[dag.edge_endpoints(first_edge).unwrap().0].id,
            0,
            "first edge source node id is expected to be 0"
        );
        assert_eq!(
            dag[dag.edge_endpoints(first_edge).unwrap().1].id,
            1,
            "first edge target node id is expected to be 1"
        );
        assert_eq!(
            dag[dag.edge_endpoints(last_edge).unwrap().0].id,
            21,
            "last edge source node id is expected to be 21"
        );
        assert_eq!(
            dag[dag.edge_endpoints(last_edge).unwrap().1].id,
            18,
            "last edge target node id is expected to be 18"
        );
        assert_eq!(dag[first_edge], 0, "first edge weight is expected to be 0");
        assert_eq!(dag[last_edge], 0, "last edge weight is expected to be 0");
    }

    #[test]
    fn test_create_dag_from_yaml_fan_in_fan_out() {
        let dag = create_dag_from_yaml("tests/sample_dags/fan_in_fan_out_format.yaml");
        let first_node = dag.node_indices().next().unwrap();
        let last_node = dag.node_indices().last().unwrap();
        let first_edge = dag.edge_indices().next().unwrap();
        let last_edge = dag.edge_indices().last().unwrap();

        assert_eq!(dag.node_count(), 20, "number of nodes is expected to be 20");
        assert_eq!(
            dag[first_node].params.get("Weight").unwrap(),
            &4,
            "first node weight is expected to be 4"
        );
        assert_eq!(
            dag[last_node].params.get("Weight").unwrap(),
            &1,
            "last node weight is expected to be 1"
        );
        assert_eq!(
            dag[first_node].params.get("execution_time").unwrap(),
            &3,
            "first node execution time is expected to be 3"
        );
        assert_eq!(
            dag[last_node].params.get("execution_time").unwrap(),
            &43,
            "last node execution time is expected to be 43"
        );
        assert_eq!(dag.edge_count(), 29, "number of edges is expected to be 29");
        assert_eq!(
            dag[dag.edge_endpoints(first_edge).unwrap().0].id,
            0,
            "first edge source node id is expected to be 0"
        );
        assert_eq!(
            dag[dag.edge_endpoints(first_edge).unwrap().1].id,
            1,
            "first edge target node id is expected to be 1"
        );
        assert_eq!(
            dag[dag.edge_endpoints(last_edge).unwrap().0].id,
            18,
            "last edge source node id is expected to be 18"
        );
        assert_eq!(
            dag[dag.edge_endpoints(last_edge).unwrap().1].id,
            19,
            "last edge target node id is expected to be 19"
        );
        assert_eq!(
            dag[first_edge], 11,
            "first edge weight is expected to be 11"
        );
        assert_eq!(dag[last_edge], 2, "last edge weight is expected to be 2");
    }

    #[test]
    fn test_create_dag_from_yaml_gnp() {
        let dag = create_dag_from_yaml("tests/sample_dags/gnp_format.yaml");
        let first_node = dag.node_indices().next().unwrap();
        let last_node = dag.node_indices().last().unwrap();
        let first_edge = dag.edge_indices().next().unwrap();
        let last_edge = dag.edge_indices().last().unwrap();

        assert_eq!(dag.node_count(), 70, "number of nodes is expected to be 70");
        assert_eq!(dag[first_node].id, 0, "first node id is expected to be 0");
        assert_eq!(dag[last_node].id, 69, "last node id is expected to be 69");
        assert_eq!(
            dag[first_node].params.get("Weight").unwrap(),
            &1,
            "first node weight is expected to be 1"
        );
        assert_eq!(
            dag[last_node].params.get("Weight").unwrap(),
            &5,
            "last node weight is expected to be 5"
        );
        assert_eq!(
            dag[first_node].params.get("execution_time").unwrap(),
            &34,
            "first node execution time is expected to be 34"
        );
        assert_eq!(
            dag[last_node].params.get("execution_time").unwrap(),
            &1,
            "last node execution time is expected to be 1"
        );
        assert_eq!(
            dag[first_node].params.get("offset").unwrap(),
            &4,
            "first node offset is expected to be 4"
        );
        assert_eq!(
            dag[last_node].params.get("offset").unwrap(),
            &5,
            "last node offset is expected to be 5"
        );
        assert_eq!(
            dag[first_node].params.get("period").unwrap(),
            &6000,
            "first node period is expected to be 6000"
        );
        assert_eq!(
            dag[last_node].params.get("period").unwrap(),
            &10,
            "last node period is expected to be 10"
        );
        assert_eq!(
            dag.edge_count(),
            233,
            "number of edges is expected to be 233"
        );
        assert_eq!(
            dag[dag.edge_endpoints(first_edge).unwrap().0].id,
            0,
            "first edge source node id is expected to be 0"
        );
        assert_eq!(
            dag[dag.edge_endpoints(first_edge).unwrap().1].id,
            2,
            "first edge target node id is expected to be 2"
        );
        assert_eq!(
            dag[dag.edge_endpoints(last_edge).unwrap().0].id,
            68,
            "last edge source node id is expected to be 68"
        );
        assert_eq!(
            dag[dag.edge_endpoints(last_edge).unwrap().1].id,
            14,
            "last edge target node id is expected to be 14"
        );
        assert_eq!(dag[first_edge], 0, "first edge weight is expected to be 0");
        assert_eq!(dag[last_edge], 0, "last edge weight is expected to be 0");
    }

    #[test]
    fn test_create_dag_from_yaml_float_params() {
        let dag = create_dag_from_yaml("tests/sample_dags/float_params.yaml");
        let first_node = dag.node_indices().next().unwrap();
        let last_node = dag.node_indices().last().unwrap();
        let first_edge = dag.edge_indices().next().unwrap();
        let last_edge = dag.edge_indices().last().unwrap();

        assert_eq!(dag.node_count(), 3, "number of nodes is expected to be 3");
        assert_eq!(
            dag[first_node].params.get("Weight").unwrap(),
            &41,
            "first node weight is expected to be 41"
        );
        assert_eq!(
            dag[last_node].params.get("Weight").unwrap(),
            &10,
            "last node weight is expected to be 10"
        );
        assert_eq!(
            dag[first_node].params.get("execution_time").unwrap(),
            &31,
            "first node execution time is expected to be 31"
        );
        assert_eq!(
            dag[last_node].params.get("execution_time").unwrap(),
            &430,
            "last node execution time is expected to be 430"
        );
        assert_eq!(dag.edge_count(), 2, "number of edges is expected to be 2");
        assert_eq!(
            dag[dag.edge_endpoints(first_edge).unwrap().0].id,
            0,
            "first edge source node id is expected to be 0"
        );
        assert_eq!(
            dag[dag.edge_endpoints(first_edge).unwrap().1].id,
            1,
            "first edge target node id is expected to be 1"
        );
        assert_eq!(
            dag[dag.edge_endpoints(last_edge).unwrap().0].id,
            1,
            "last edge source node id is expected to be 1"
        );
        assert_eq!(
            dag[dag.edge_endpoints(last_edge).unwrap().1].id,
            2,
            "last edge target node id is expected to be 19"
        );
        assert_eq!(
            dag[first_edge], 111,
            "first edge weight is expected to be 111"
        );
        assert_eq!(dag[last_edge], 20, "last edge weight is expected to be 20");
    }

    #[test]
    #[should_panic]
    fn test_create_dag_from_yaml_path() {
        let _dag = create_dag_from_yaml("tests/sample_dags/disable_path.yaml");
    }

    #[test]
    #[should_panic]
    fn test_create_dag_from_yaml_no_yaml() {
        let _dag = create_dag_from_yaml("tests/sample_dags/no_yaml.tex");
    }

    #[test]
    #[should_panic]
    fn test_create_dag_from_yaml_broken_link() {
        let _dag = create_dag_from_yaml("tests/sample_dags/broken_link.yaml");
    }
}
