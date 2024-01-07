use aiken_lang::ast::Tracing;
use aiken_project::Project;
use aiken_project::telemetry::Terminal;

const MINT_NFT_PROJECT: &str = "./aiken/mint_nft";

fn build_project(path: &str) {
    let mut project = Project::new(path.into(), Terminal::default())
        .expect(&format!("Project not found: {:?}", path));
    let build_result = project.build(false, Tracing::KeepTraces);

    if let Err(err) = build_result {
        err.iter().for_each(|e| e.report());
        panic!("🍂 Failed to build Aiken code at {}🍂", path);
    }
}

fn main() {
    build_project(MINT_NFT_PROJECT);
}
