use log::warn;
use petgraph::Graph;
use serde_derive::{Deserialize, Serialize};

use crate::{
    graph_extension::{GraphExtension, NodeData},
    output_log::append_info_to_yaml,
};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DAGSetInfo {
    total_utilization: f32,
    each_dag_info: Vec<DAGInfo>,
}

impl DAGSetInfo {
    pub fn new(dag_set: &[Graph<NodeData, i32>]) -> Self {
        let mut total_utilization = 0.0;
        let mut each_dag_info = Vec::new();

        for dag in dag_set.iter() {
            let dag_info = DAGInfo::new(dag);
            total_utilization += dag_info.get_utilization();
            each_dag_info.push(dag_info);
        }

        Self {
            total_utilization,
            each_dag_info,
        }
    }

    pub fn get_total_utilization(&self) -> f32 {
        self.total_utilization
    }

    pub fn get_each_dag_info(&self) -> Vec<DAGInfo> {
        self.each_dag_info.clone()
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DAGInfo {
    critical_path_length: i32,
    period: i32,
    end_to_end_deadline: i32,
    volume: i32,
    utilization: f32,
}

impl DAGInfo {
    pub fn new(dag: &Graph<NodeData, i32>) -> Self {
        let period = dag.get_head_period().unwrap_or(0);
        let end_to_end_deadline = dag.get_end_to_end_deadline().unwrap_or(0);
        let volume = dag.get_volume();
        let utilization = match (end_to_end_deadline, period) {
            (0, 0) => {
                warn!("Both period and end_to_end_deadline are not set.");
                0.0
            }
            (_, 0) => {
                warn!("Period is not set.");
                0.0
            }
            (0, _) => period as f32 / volume as f32,
            (_, _) => period as f32 / volume as f32,
        };

        let critical_path = dag.clone().get_critical_path();
        Self {
            critical_path_length: dag.get_total_wcet_from_nodes(&critical_path),
            period,
            end_to_end_deadline,
            volume,
            utilization,
        }
    }

    pub fn get_critical_path_length(&self) -> i32 {
        self.critical_path_length
    }

    pub fn get_period(&self) -> i32 {
        self.period
    }

    pub fn get_volume(&self) -> i32 {
        self.volume
    }

    pub fn get_utilization(&self) -> f32 {
        self.utilization
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ProcessorInfo {
    number_of_cores: usize,
}

impl ProcessorInfo {
    pub fn new(number_of_cores: usize) -> Self {
        Self { number_of_cores }
    }

    pub fn get_number_of_cores(&self) -> usize {
        self.number_of_cores
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DAGSetLog {
    dag_set_log: Vec<DAGLog>,
}

impl DAGSetLog {
    pub fn new(dag_set: &[Graph<NodeData, i32>]) -> Self {
        let mut dag_set_log = Vec::with_capacity(dag_set.len());
        for i in 0..dag_set.len() {
            dag_set_log.push(DAGLog::new(i));
        }
        Self { dag_set_log }
    }

    pub fn get_dag_set_log(&self) -> Vec<DAGLog> {
        self.dag_set_log.clone()
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DAGLog {
    dag_id: usize,
    release_time: i32,
    start_time: i32,
    finish_time: i32,
}

impl DAGLog {
    pub fn new(dag_id: usize) -> Self {
        Self {
            dag_id,
            release_time: Default::default(),
            start_time: Default::default(),
            finish_time: Default::default(),
        }
    }

    pub fn get_dag_id(&self) -> usize {
        self.dag_id
    }

    pub fn get_release_time(&self) -> i32 {
        self.release_time
    }

    pub fn get_start_time(&self) -> i32 {
        self.start_time
    }

    pub fn get_finish_time(&self) -> i32 {
        self.finish_time
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct NodeLog {
    core_id: usize,
    dag_id: usize, // Used to distinguish DAGs when the scheduler input is DAGSet
    node_id: usize,
    start_time: i32,
    finish_time: i32,
}

impl NodeLog {
    pub fn new(dag_id: usize, node_id: usize) -> Self {
        Self {
            core_id: Default::default(),
            dag_id,
            node_id,
            start_time: Default::default(),
            finish_time: Default::default(),
        }
    }

    pub fn get_core_id(&self) -> usize {
        self.core_id
    }

    pub fn get_dag_id(&self) -> usize {
        self.dag_id
    }

    pub fn get_node_id(&self) -> usize {
        self.node_id
    }

    pub fn get_start_time(&self) -> i32 {
        self.start_time
    }

    pub fn get_finish_time(&self) -> i32 {
        self.finish_time
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct NodeLogs {
    node_logs: Vec<NodeLog>,
}

impl NodeLogs {
    pub fn new(dag: &Graph<NodeData, i32>) -> Self {
        let mut node_logs = Vec::with_capacity(dag.node_count());
        for node in dag.node_indices() {
            node_logs.push(NodeLog::new(0, dag[node].id as usize));
        }
        Self { node_logs }
    }

    pub fn get_node_logs(&self) -> Vec<NodeLog> {
        self.node_logs.clone()
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct NodeSetLogs {
    node_set_logs: Vec<Vec<NodeLog>>,
}

impl NodeSetLogs {
    pub fn new(dag_set: &[Graph<NodeData, i32>]) -> Self {
        let mut node_set_logs = Vec::with_capacity(dag_set.len());
        for (i, dag) in dag_set.iter().enumerate() {
            let mut node_logs = Vec::with_capacity(dag.node_count());
            for node in dag.node_indices() {
                node_logs.push(NodeLog::new(i, dag[node].id as usize));
            }
            node_set_logs.push(node_logs);
        }

        Self { node_set_logs }
    }

    pub fn get_node_set_logs(&self) -> Vec<Vec<NodeLog>> {
        self.node_set_logs.clone()
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ProcessorLog {
    average_utilization: f32,
    variance_utilization: f32,
    core_logs: Vec<CoreLog>,
}

impl ProcessorLog {
    pub fn new(num_cores: usize) -> Self {
        Self {
            average_utilization: Default::default(),
            variance_utilization: Default::default(),
            core_logs: (0..num_cores).map(CoreLog::new).collect(),
        }
    }

    pub fn get_average_utilization(&self) -> f32 {
        self.average_utilization
    }

    pub fn get_variance_utilization(&self) -> f32 {
        self.variance_utilization
    }

    pub fn get_core_logs(&self) -> Vec<CoreLog> {
        self.core_logs.clone()
    }

    fn calculate_average_utilization(&mut self) {
        self.average_utilization = self
            .core_logs
            .iter()
            .map(|core_log| core_log.utilization)
            .sum::<f32>()
            / self.core_logs.len() as f32;
    }

    fn calculate_variance_utilization(&mut self) {
        self.variance_utilization = self
            .core_logs
            .iter()
            .map(|core_log| (core_log.utilization - self.average_utilization).powi(2))
            .sum::<f32>()
            / self.core_logs.len() as f32;
    }

    fn calculate_cores_utilization(&mut self, schedule_length: i32) {
        for core_log in self.core_logs.iter_mut() {
            core_log.calculate_utilization(schedule_length);
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct CoreLog {
    core_id: usize,
    total_proc_time: i32,
    utilization: f32,
}

impl CoreLog {
    fn new(core_id: usize) -> Self {
        Self {
            core_id,
            total_proc_time: Default::default(),
            utilization: Default::default(),
        }
    }

    pub fn get_core_id(&self) -> usize {
        self.core_id
    }

    pub fn get_total_proc_time(&self) -> i32 {
        self.total_proc_time
    }

    pub fn get_utilization(&self) -> f32 {
        self.utilization
    }

    fn calculate_utilization(&mut self, schedule_length: i32) {
        self.utilization = self.total_proc_time as f32 / schedule_length as f32;
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DAGSchedulerLog {
    dag_info: DAGInfo,
    processor_info: ProcessorInfo,
    node_logs: NodeLogs,
    processor_log: ProcessorLog,
}

impl DAGSchedulerLog {
    pub fn new(dag: &Graph<NodeData, i32>, num_cores: usize) -> Self {
        Self {
            dag_info: DAGInfo::new(dag),
            processor_info: ProcessorInfo::new(num_cores),
            node_logs: NodeLogs::new(dag),
            processor_log: ProcessorLog::new(num_cores),
        }
    }

    pub fn get_dag_info(&self) -> DAGInfo {
        self.dag_info.clone()
    }

    pub fn get_processor_info(&self) -> ProcessorInfo {
        self.processor_info.clone()
    }

    pub fn get_node_logs(&self) -> NodeLogs {
        self.node_logs.clone()
    }

    pub fn get_processor_log(&self) -> ProcessorLog {
        self.processor_log.clone()
    }

    pub fn set_dag_info(&mut self, dag_info: DAGInfo) {
        self.dag_info = dag_info;
    }

    pub fn set_processor_info(&mut self, processor_info: ProcessorInfo) {
        self.processor_info = processor_info;
    }

    pub fn set_node_logs(&mut self, node_logs: NodeLogs) {
        self.node_logs = node_logs;
    }

    pub fn set_processor_log(&mut self, processor_log: ProcessorLog) {
        self.processor_log = processor_log;
    }

    pub fn write_allocating_log(
        &mut self,
        node_data: &NodeData,
        core_id: usize,
        current_time: i32,
    ) {
        let node_id = node_data.id as usize;
        self.node_logs.node_logs[node_id].core_id = core_id;
        self.node_logs.node_logs[node_id].start_time = current_time;
        self.processor_log.core_logs[core_id].total_proc_time +=
            node_data.params.get("execution_time").unwrap_or(&0);
    }

    pub fn write_finishing_node_log(&mut self, node_data: &NodeData, current_time: i32) {
        self.node_logs.node_logs[node_data.id as usize].finish_time = current_time;
    }

    pub fn write_scheduling_log(&mut self, schedule_length: i32) {
        self.processor_log
            .calculate_cores_utilization(schedule_length);
        self.processor_log.calculate_average_utilization();
        self.processor_log.calculate_variance_utilization();
    }

    pub fn dump_log_to_yaml(&self, file_path: &str) {
        let yaml = serde_yaml::to_string(&self).expect("Failed to serialize DAGInfo to YAML");
        append_info_to_yaml(file_path, &yaml);
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DAGSetSchedulerLog {
    dag_set_info: DAGSetInfo,
    processor_info: ProcessorInfo,
    dag_set_log: DAGSetLog,
    node_set_logs: NodeSetLogs,
    processor_log: ProcessorLog,
}

impl DAGSetSchedulerLog {
    pub fn new(dag_set: &[Graph<NodeData, i32>], num_cores: usize) -> Self {
        Self {
            dag_set_info: DAGSetInfo::new(dag_set),
            processor_info: ProcessorInfo::new(num_cores),
            dag_set_log: DAGSetLog::new(dag_set),
            node_set_logs: NodeSetLogs::new(dag_set),
            processor_log: ProcessorLog::new(num_cores),
        }
    }

    pub fn get_dag_set_info(&self) -> DAGSetInfo {
        self.dag_set_info.clone()
    }

    pub fn get_processor_info(&self) -> ProcessorInfo {
        self.processor_info.clone()
    }

    pub fn get_dag_set_log(&self) -> DAGSetLog {
        self.dag_set_log.clone()
    }

    pub fn get_node_set_logs(&self) -> NodeSetLogs {
        self.node_set_logs.clone()
    }

    pub fn get_processor_log(&self) -> ProcessorLog {
        self.processor_log.clone()
    }

    pub fn write_dag_release_time_log(&mut self, dag_id: usize, release_time: i32) {
        self.dag_set_log.dag_set_log[dag_id].release_time = release_time;
    }

    pub fn write_dag_start_time_log(&mut self, dag_id: usize, start_time: i32) {
        self.dag_set_log.dag_set_log[dag_id].start_time = start_time;
    }

    pub fn write_dag_finish_time_log(&mut self, dag_id: usize, finish_time: i32) {
        self.dag_set_log.dag_set_log[dag_id].finish_time = finish_time;
    }

    pub fn write_allocating_log(
        &mut self,
        dag_id: usize,
        node_id: usize,
        core_id: usize,
        current_time: i32,
        proc_time: i32,
    ) {
        self.node_set_logs.node_set_logs[dag_id][node_id].core_id = core_id;
        self.node_set_logs.node_set_logs[dag_id][node_id].start_time = current_time;
        self.processor_log.core_logs[core_id].total_proc_time += proc_time;
    }

    pub fn write_finishing_node_log(&mut self, node_data: &NodeData, finish_time: i32) {
        self.node_set_logs.node_set_logs[node_data.params["dag_id"] as usize]
            [node_data.id as usize]
            .finish_time = finish_time;
    }

    pub fn write_scheduling_log(&mut self, schedule_length: i32) {
        self.processor_log
            .calculate_cores_utilization(schedule_length);
        self.processor_log.calculate_average_utilization();
        self.processor_log.calculate_variance_utilization();
    }

    pub fn dump_log_to_yaml(&self, file_path: &str) {
        let yaml = serde_yaml::to_string(&self).expect("Failed to serialize DAGInfo to YAML");
        append_info_to_yaml(file_path, &yaml);
    }
}