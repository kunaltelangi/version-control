use std::fs;
use std::path::Path;

pub async fn run() -> anyhow::Result<()> {
    let vcs_dir = Path::new(".vcs");
    if vcs_dir.exists() {
        println!("Repository already initialized.");
    } else {
        fs::create_dir(vcs_dir)?;
        println!("Initialized empty VCS repository in .vcs/");
    }
    Ok(())
}
