use super::*;
use std::fs;
use walkdir::WalkDir;

pub fn execute() -> Result<()> {
    let repo_root = get_repo_root()?;
    let config = read_config()?;
    let index = read_index()?;
    let kvcs_dir = get_kvcs_dir()?;
    
    println!("KVCS Repository Information");
    println!("==========================");
    println!();
    
    println!("Repository path: {}", repo_root.display());
    println!("KVCS directory: {}", kvcs_dir.display());
    println!();
    
    println!("Branch Information:");
    println!("  Current branch: {}", config.current_branch);
    println!("  Total branches: {}", config.branches.len());
    for (branch, commit_hash) in &config.branches {
        let status = if commit_hash.is_empty() { 
            "(no commits)".to_string() 
        } else { 
            format!("({})", &commit_hash[..8])
        };
        
        if branch == &config.current_branch {
            println!("  * {} {}", branch, status);
        } else {
            println!("    {} {}", branch, status);
        }
    }
    println!();
    
    let commit_count = count_commits()?;
    println!("Commit Information:");
    println!("  Total commits: {}", commit_count);
    
    if let Some(current_commit) = get_current_commit_hash()? {
        let commit_data = read_object(&current_commit)?;
        let commit: Commit = serde_json::from_slice(&commit_data)?;
        println!("  Latest commit: {} ({})", &current_commit[..8], commit.message);
        println!("  Author: {}", commit.author);
        println!("  Date: {}", commit.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    } else {
        println!("  Latest commit: None");
    }
    println!();
    
    // File statistics
    let (tracked_files, modified_files, untracked_files) = analyze_files(&repo_root, &index)?;
    println!("File Statistics:");
    println!("  Tracked files: {}", tracked_files);
    println!("  Modified files: {}", modified_files);
    println!("  Untracked files: {}", untracked_files);
    println!("  Staged files: {}", index.files.len());
    println!();
    
    let repo_size = calculate_repo_size(&kvcs_dir)?;
    println!("Repository Size:");
    println!("  .kvcs directory: {}", format_bytes(repo_size));
    
    let objects_count = count_objects(&kvcs_dir)?;
    println!("  Objects stored: {}", objects_count);
    println!();
    
    // Stash information
    let stash = read_stash()?;
    println!("Stash Information:");
    println!("  Stash entries: {}", stash.entries.len());
    if !stash.entries.is_empty() {
        println!("  Latest stash: {}", stash.entries[0].message);
    }
    println!();
    
    // Configuration
    println!("Configuration:");
    println!("  User name: {}", config.user_name);
    println!("  User email: {}", config.user_email);
    println!("  Remotes: {}", config.remotes.len());
    
    Ok(())
}

fn count_commits() -> Result<usize> {
    let mut count = 0;
    let mut current_commit = get_current_commit_hash()?;
    
    while let Some(commit_hash) = current_commit {
        count += 1;
        let commit_data = read_object(&commit_hash)?;
        let commit: Commit = serde_json::from_slice(&commit_data)?;
        current_commit = commit.parent;
    }
    
    Ok(count)
}

fn analyze_files(repo_root: &Path, index: &Index) -> Result<(usize, usize, usize)> {
    let mut working_files = std::collections::HashSet::new();
    let mut modified_count = 0;
    for entry in WalkDir::new(repo_root) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && !path.starts_with(repo_root.join(".kvcs")) {
            let relative_path = path.strip_prefix(repo_root)?
                .to_string_lossy()
                .replace('\\', "/");
            working_files.insert(relative_path);
        }
    }
    
    for (file_path, index_entry) in &index.files {
        let full_path = repo_root.join(file_path);
        if full_path.exists() {
            let content = fs::read(&full_path)?;
            let current_hash = hash_content(&content);
            if current_hash != index_entry.hash {
                modified_count += 1;
            }
        }
    }
    
    let tracked_files = index.files.len();
    let untracked_files = working_files.len() - tracked_files;
    
    Ok((tracked_files, modified_count, untracked_files))
}

fn calculate_repo_size(kvcs_dir: &Path) -> Result<u64> {
    let mut total_size = 0;
    
    for entry in WalkDir::new(kvcs_dir) {
        let entry = entry?;
        if entry.path().is_file() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }
    }
    
    Ok(total_size)
}

fn count_objects(kvcs_dir: &Path) -> Result<usize> {
    let objects_dir = kvcs_dir.join("objects");
    let mut count = 0;
    
    if objects_dir.exists() {
        for entry in WalkDir::new(objects_dir) {
            let entry = entry?;
            if entry.path().is_file() {
                count += 1;
            }
        }
    }
    
    Ok(count)
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}
