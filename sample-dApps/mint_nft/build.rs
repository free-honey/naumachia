use aiken::Terminal;
use aiken_project::script::EvalInfo;
use aiken_project::{pretty, telemetry, Project};
use std::collections::BTreeMap;

use owo_colors::OwoColorize;
use uplc::machine::cost_model::ExBudget;

const PROJECT: &str = "./mint_nft";

fn main() {
    let mut project = Project::new(PROJECT.into(), Terminal::default())
        .expect(&format!("Project not found: {:?}", PROJECT));
    let build_result = project.build(false);

    if let Err(err) = build_result {
        err.report();
        panic!("🍂 Failed to build Aiken code 🍂");
    }
}
