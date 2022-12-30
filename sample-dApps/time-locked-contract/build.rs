use aiken::Terminal;
use aiken_project::Project;

const PROJECT: &str = "./time_locked";

fn main() {
    let mut project = Project::new(PROJECT.into(), Terminal::default())
        .expect(&format!("Project not found: {:?}", PROJECT));
    let build_result = project.build(false);

    if let Err(err) = build_result {
        err.report();
        panic!("🍂 Failed to build Aiken code 🍂");
    }
}
