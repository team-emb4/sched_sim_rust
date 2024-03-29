mod dynfed;

use clap::Parser;
use dynfed::DynamicFederatedScheduler;
use lib::{
    dag_creator::create_dag_set_from_dir,
    dag_set_scheduler::{DAGSetSchedulerBase, PreemptiveType},
    fixed_priority_scheduler::FixedPriorityScheduler,
    graph_extension::GraphExtension,
    homogeneous::HomogeneousProcessor,
    log::dump_dag_set_scheduler_result_to_yaml,
    processor::ProcessorBase,
    util::{adjust_to_implicit_deadline, load_yaml},
};

#[derive(Parser)]
#[clap(
    name = "DynFed_Algorithm",
    version = "1.0",
    about = "About:
    DynFed_Algorithm operates on the same assumption of period and end_to_end_deadline.
    Therefore, the period shall be considered as the end_to_end_deadline.
    If there is no period, the end_to_end_deadline shall be obtained."
)]
struct ArgParser {
    ///Path to DAGSet directory.
    #[clap(short = 'd', long = "dag_dir_path", required = true)]
    dag_dir_path: String,
    ///Number of processing cores.
    #[clap(short = 'c', long = "number_of_cores", required = true)]
    number_of_cores: usize,
    ///Path to output directory.
    #[clap(short = 'o', long = "output_dir_path", default_value = "../outputs")]
    output_dir_path: String,
}

fn main() {
    let arg: ArgParser = ArgParser::parse();

    let mut dag_set = create_dag_set_from_dir(&arg.dag_dir_path);
    adjust_to_implicit_deadline(&mut dag_set);

    let homogeneous_processor = HomogeneousProcessor::new(arg.number_of_cores);
    let mut dynfed_scheduler: DynamicFederatedScheduler<
        FixedPriorityScheduler<HomogeneousProcessor>,
    > = DynamicFederatedScheduler::new(&dag_set, &homogeneous_processor);

    dynfed_scheduler.schedule(PreemptiveType::NonPreemptive);
    let file_path = dynfed_scheduler.dump_log(&arg.output_dir_path, "FixedPriority");

    // Check the result
    let yaml_doc = &load_yaml(&file_path)[0];
    let dag_set_log = &yaml_doc["dag_set_log"];
    let mut result = true;
    for dag in dag_set {
        if dag_set_log[dag.get_dag_param("dag_id") as usize]["worst_response_time"]
            .as_i64()
            .unwrap()
            > dag.get_head_period().unwrap() as i64
        {
            result = false;
            break;
        }
    }

    dump_dag_set_scheduler_result_to_yaml(&file_path, result);
}
