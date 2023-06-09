use clap::Parser;
use lib::dag_creator::*;
use lib::homogeneous;
use lib::output_log::*;
use lib::processor::ProcessorBase;
use outputs_result::*;
mod federated;
mod outputs_result;

/// Application description and arguments definition using clap crate
#[derive(Parser)]
#[clap()]

/// Application arguments definition using clap crate
struct AppArg {
    #[clap(short = 'f', long = "dag_file_path", required = false)]
    dag_file_path: Option<String>,
    #[clap(short = 'd', long = "dag_dir_path", required = false)]
    dag_dir_path: Option<String>,
    #[clap(short = 'c', long = "number_of_cores", required = true)]
    number_of_cores: usize,
    #[clap(short = 'o', long = "output_dir_path", default_value = "../outputs")]
    output_dir_path: String,
}

/// Application main function
fn main() {
    let arg: AppArg = AppArg::parse();
    if let Some(dag_dir_path) = arg.dag_dir_path {
        let number_of_cores = arg.number_of_cores;
        let mut dag_set = create_dag_set_from_dir(&dag_dir_path);
        let result = federated::federated(&mut dag_set, number_of_cores);
        let file_path = create_scheduler_log_yaml_file(&arg.output_dir_path, "federated");
        let homogeneous_processor = homogeneous::HomogeneousProcessor::new(number_of_cores);
        dump_dag_set_info_to_yaml(&file_path, dag_set);
        dump_processor_info_to_yaml(&file_path, homogeneous_processor);
        dump_federated_result_to_file(&file_path, result);
    }
}
