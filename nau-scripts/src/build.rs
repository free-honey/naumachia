use aiken::Terminal;
use aiken_project::Project;

const MINT_NFT_PROJECT: &str = "./aiken/mint_nft";

fn build_project(path: &str) {
    let mut project = Project::new(path.into(), Terminal::default())
        .expect(&format!("Project not found: {:?}", PROJECT));
    let build_result = project.build(false);

    if let Err(err) = build_result {
        err.report();
        panic!("🍂 Failed to build Aiken code at {}🍂", path);
    }
}

fn main() {
    build_project(MINT_NFT_PROJECT);
}
