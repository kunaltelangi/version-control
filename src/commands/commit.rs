use std::fs;
use std::path::Path;
use chrono::Utc;

fn is_repo_initialized() -> bool {
    Path::new(".vcs").exists()
}

fn get_commit_id() -> String {
    Utc::now().format("%Y%m%d%H%M%S").to_string()
}

pub async fn run(message: String) -> anyhow::Result<()> {
    if !is_repo_initialized() {
        println!("No VCS repository found. Run `vcs init` first.");
        return Ok(());
    }

    let index_dir = Path::new(".vcs/index");
    if !index_dir.exists() || fs::read_dir(index_dir)?.next().is_none() {
        println!("No changes staged for commit.");
        return Ok(());
    }

    let commit_id = get_commit_id();
    let commit_dir = Path::new(".vcs/commits").join(&commit_id);
    fs::create_dir_all(&commit_dir)?;

    // Copy staged files to commit directory
    for entry in fs::read_dir(index_dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let dest = commit_dir.join(&file_name);
        fs::copy(entry.path(), dest)?;
    }

    // Save commit message
    fs::write(commit_dir.join("message.txt"), message)?;

    // Clear staging area
    fs::remove_dir_all(index_dir)?;
    fs::create_dir_all(index_dir)?;

    println!("Committed as {}", commit_id);
    Ok(())
}
