use std::fs;
use std::path::Path;

fn is_repo_initialized() -> bool {
    Path::new(".vcs").exists()
}

pub async fn run(file: String) -> anyhow::Result<()> {
    if !is_repo_initialized() {
        println!("❌ No VCS repository found. Run `vcs init` first.");
        return Ok(());
    }

    let vcs_dir = Path::new(".vcs");
    let index_dir = vcs_dir.join("index");
    fs::create_dir_all(&index_dir)?;

    let src = Path::new(&file);
    if !src.exists() {
        println!("❌ File does not exist: {}", file);
        return Ok(());
    }

    let dest = index_dir.join(&file);

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    if dest.exists() {
        println!("⚠️  Warning: File '{}' already staged. Overwriting...", file);
    }

    fs::copy(&src, &dest)?;
    println!("✅ Added '{}' to staging area.", file);

    Ok(())
}
