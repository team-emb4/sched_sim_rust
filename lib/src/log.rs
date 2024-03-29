use crate::graph_extension::{GraphExtension, NodeData};
use crate::util::append_info_to_yaml;
use log::warn;
use petgraph::Graph;
use serde::Serialize;
use serde_derive::{Deserialize, Serialize};

pub fn dump_struct(file_path: &str, target_struct: &impl Serialize) {
    let yaml = serde_yaml::to_string(&target_struct).expect("Failed to serialize.");
    append_info_to_yaml(file_path, &yaml);
}

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

    fn get_utilization(&self) -> f32 {
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
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DAGLog {
    dag_id: usize,
    release_time: Vec<i32>,
    finish_time: Vec<i32>,
    response_time: Vec<i32>,
    average_response_time: f32,
    worst_response_time: i32,
}

impl DAGLog {
    pub fn new(dag_id: usize) -> Self {
        Self {
            dag_id,
            release_time: Default::default(),
            finish_time: Default::default(),
            response_time: Default::default(),
            average_response_time: Default::default(),
            worst_response_time: Default::default(),
        }
    }

    pub fn calculate_response_time(&mut self) {
        // Unequal lengths indicate that the DAG was not completed within the hyper_period, and deadline miss occurred.
        if self.release_time.len() != self.finish_time.len() {
            // Mark as a deadline miss by maximizing the response time.
            self.finish_time.push(std::i32::MAX);
        }
        self.response_time = self
            .release_time
            .iter()
            .zip(self.finish_time.iter())
            .map(|(release_time, finish_time)| *finish_time - *release_time)
            .collect();
    }

    pub fn calculate_average_response_time(&mut self) {
        self.average_response_time =
            self.response_time.iter().sum::<i32>() as f32 / self.response_time.len() as f32;
    }

    pub fn calculate_worst_response_time(&mut self) {
        self.worst_response_time = *self.response_time.iter().max().unwrap();
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum JobEventTimes {
    StartTime(i32),
    ResumeTime(i32),
    FinishTime(i32),
    PreemptedTime(i32),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JobLog {
    core_id: usize,
    dag_id: usize, // Used to distinguish DAGs when the scheduler input is DAGSet
    node_id: usize,
    job_id: usize,
    event_time: JobEventTimes,
}

impl JobLog {
    #[allow(clippy::too_many_arguments)]
    fn new(
        core_id: usize,
        dag_id: usize,
        node_id: usize,
        job_id: usize,
        event_time: JobEventTimes,
    ) -> Self {
        Self {
            core_id,
            dag_id,
            node_id,
            job_id,
            event_time,
        }
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

    fn calculate_utilization(&mut self, schedule_length: i32) {
        self.utilization = self.total_proc_time as f32 / schedule_length as f32;
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DAGSchedulerLog {
    dag_info: DAGInfo,
    processor_info: ProcessorInfo,
    node_logs: Vec<JobLog>,
    processor_log: ProcessorLog,
}

impl DAGSchedulerLog {
    pub fn new(dag: &Graph<NodeData, i32>, num_cores: usize) -> Self {
        Self {
            dag_info: DAGInfo::new(dag),
            processor_info: ProcessorInfo::new(num_cores),
            node_logs: Vec::new(),
            processor_log: ProcessorLog::new(num_cores),
        }
    }

    pub fn write_allocating_job(
        &mut self,
        node_data: &NodeData,
        core_id: usize,
        current_time: i32,
    ) {
        let job_log = JobLog::new(
            core_id,
            0, // This is a fixed value because DAG is only one.
            node_data.id as usize,
            0, // This is a fixed value because DAG is released only once.
            JobEventTimes::StartTime(current_time),
        );
        self.node_logs.push(job_log);
    }

    pub fn write_processing_time(&mut self, core_indices: &[usize]) {
        for core_index in core_indices {
            self.processor_log.core_logs[*core_index].total_proc_time += 1;
        }
    }

    pub fn write_finishing_job(&mut self, node_data: &NodeData, core_id: usize, current_time: i32) {
        let job_log = JobLog::new(
            core_id,
            0, // This is a fixed value because DAG is only one.
            node_data.id as usize,
            0, // This is a fixed value because DAG is released only once.
            JobEventTimes::FinishTime(current_time),
        );
        self.node_logs.push(job_log);
    }

    pub fn calculate_utilization(&mut self, schedule_length: i32) {
        self.processor_log
            .calculate_cores_utilization(schedule_length);
        self.processor_log.calculate_average_utilization();
        self.processor_log.calculate_variance_utilization();
    }

    pub fn dump_log_to_yaml(&self, file_path: &str) {
        dump_struct(file_path, self);
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DAGSetSchedulerLog {
    dag_set_info: DAGSetInfo,
    processor_info: ProcessorInfo,
    dag_set_log: Vec<DAGLog>,
    node_set_logs: Vec<Vec<JobLog>>,
    processor_log: ProcessorLog,
}

impl DAGSetSchedulerLog {
    pub fn new(dag_set: &[Graph<NodeData, i32>], num_cores: usize) -> Self {
        let mut dag_set_log = Vec::with_capacity(dag_set.len());
        for i in 0..dag_set.len() {
            dag_set_log.push(DAGLog::new(i));
        }

        Self {
            dag_set_info: DAGSetInfo::new(dag_set),
            processor_info: ProcessorInfo::new(num_cores),
            dag_set_log,
            node_set_logs: vec![Vec::new(); dag_set.len()],
            processor_log: ProcessorLog::new(num_cores),
        }
    }

    pub fn write_dag_release_time(&mut self, dag_id: usize, release_time: i32) {
        self.dag_set_log[dag_id].release_time.push(release_time);
    }

    pub fn write_dag_finish_time(&mut self, dag_id: usize, finish_time: i32) {
        self.dag_set_log[dag_id].finish_time.push(finish_time);
    }

    pub fn write_allocating_job(
        &mut self,
        node_data: &NodeData,
        core_id: usize,
        job_id: usize,
        current_time: i32,
    ) {
        if node_data.params.contains_key("is_preempted") {
            self.write_job_event(
                node_data,
                core_id,
                job_id - 1,
                JobEventTimes::ResumeTime(current_time),
            )
        } else {
            self.write_job_event(
                node_data,
                core_id,
                job_id - 1,
                JobEventTimes::StartTime(current_time),
            )
        }
    }

    pub fn write_job_event(
        &mut self,
        node_data: &NodeData,
        core_id: usize,
        job_id: usize,
        event_time: JobEventTimes,
    ) {
        let dag_id = node_data.get_params_value("dag_id") as usize;
        let job_log = JobLog::new(core_id, dag_id, node_data.id as usize, job_id, event_time);
        self.node_set_logs[dag_id].push(job_log);
    }

    pub fn write_processing_time(&mut self, core_indices: &[usize]) {
        for core_index in core_indices {
            self.processor_log.core_logs[*core_index].total_proc_time += 1;
        }
    }

    pub fn calculate_response_time(&mut self) {
        for dag_log in self.dag_set_log.iter_mut() {
            dag_log.calculate_response_time();
            dag_log.calculate_average_response_time();
            dag_log.calculate_worst_response_time();
        }
    }

    pub fn calculate_utilization(&mut self, schedule_length: i32) {
        self.processor_log
            .calculate_cores_utilization(schedule_length);
        self.processor_log.calculate_average_utilization();
        self.processor_log.calculate_variance_utilization();
    }

    pub fn dump_log_to_yaml(&self, file_path: &str) {
        dump_struct(file_path, self);
    }
}

#[derive(Serialize, Deserialize)]
struct DAGSchedulerResultInfo {
    schedule_length: i32,
    period_factor: f32,
    result: bool,
}

pub fn dump_dag_scheduler_result_to_yaml(
    file_path: &str,
    schedule_length: i32,
    period_factor: f32,
    result: bool,
) {
    let result_info = DAGSchedulerResultInfo {
        schedule_length,
        period_factor,
        result,
    };
    dump_struct(file_path, &result_info);
}

#[derive(Serialize, Deserialize)]
struct DAGSetSchedulerResultInfo {
    result: bool,
}

pub fn dump_dag_set_scheduler_result_to_yaml(file_path: &str, result: bool) {
    let result_info = DAGSetSchedulerResultInfo { result };
    dump_struct(file_path, &result_info);
}
