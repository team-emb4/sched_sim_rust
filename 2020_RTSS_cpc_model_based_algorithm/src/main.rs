mod parallel_provider_consumer;
mod prioritization_cpc_model;

use clap::Parser;
use lib::{
    dag_creator::create_dag_from_yaml, dag_scheduler::DAGSchedulerBase,
    fixed_priority_scheduler::FixedPriorityScheduler, graph_extension::GraphExtension,
    homogeneous::HomogeneousProcessor, log::dump_dag_scheduler_result_to_yaml,
    processor::ProcessorBase,
};
use log::warn;

#[derive(Parser)]
#[clap(
    name = "CPC_Model_Based_Algorithm",
    version = "1.0",
    about = "About:
    The CPC_Model_Based_Algorithm operates under the assumption of a constrained deadline.
    In essence, it presumes that the input Directed Acyclic Graph (DAG) incorporates and adheres to these constrained deadlines.
    If, however, the input DAG does not contain a predefined constrained deadline, 
    the algorithm will impose one by multiplying the period of the input DAG by an arbitrary multiplier."
)]
struct ArgParser {
    ///Path to DAG file.
    #[clap(short = 'f', long = "dag_file_path", required = true)]
    dag_file_path: String,
    ///Number of processing cores.
    #[clap(short = 'c', long = "number_of_cores", required = true)]
    number_of_cores: usize,
    ///Path to output directory.
    #[clap(short = 'o', long = "output_dir_path", default_value = "../outputs")]
    output_dir_path: String,
    ///Multiplier to compute constrained deadlines
    #[clap(short = 'r', long = "ratio_deadline_to_period", default_value = "1.0")]
    ratio_deadline_to_period: f32,
}

fn main() {
    let arg: ArgParser = ArgParser::parse();
    if arg.ratio_deadline_to_period > 1.0 {
        panic!("ratio_deadline_to_period must be less than or equal to 1.0");
    }
    let mut dag = create_dag_from_yaml(&arg.dag_file_path, false);
    let homogeneous_processor = HomogeneousProcessor::new(arg.number_of_cores);
    prioritization_cpc_model::assign_priority_to_cpc_model(&mut dag);
    let mut fixed_priority_scheduler = FixedPriorityScheduler::new(&dag, &homogeneous_processor);
    let (schedule_length, _) = fixed_priority_scheduler.schedule();
    let constrained_end_to_end_deadline = if let Some(deadline) = dag.get_end_to_end_deadline() {
        deadline as f32
    } else {
        warn!("Since the end-to-end deadline is not set in the input DAG, the end-to-end deadline is determined using ratio_deadline_to_period.");
        dag.get_head_period().unwrap() as f32 * arg.ratio_deadline_to_period
    };
    let result = (schedule_length as f32) <= constrained_end_to_end_deadline;
    let file_path = fixed_priority_scheduler.dump_log(&arg.output_dir_path, "cpc_model_based");

    dump_dag_scheduler_result_to_yaml(
        &file_path,
        schedule_length,
        arg.ratio_deadline_to_period,
        result,
    );
}
