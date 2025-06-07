use super::*;
use chrono::Utc;

pub fn execute(branch: String, no_ff: bool) -> Result<()> {
    let config = read_config()?;
    
    if branch == config.current_branch {
        return Err("Cannot merge branch into itself".into());
    }
    
    let target_commit = match config.branches.get(&branch) {
        Some(hash) if !hash.is_empty() => hash.clone(),
        _ => return Err(format!("Branch '{}' not found or has no commits", branch).into()),
    };
    
    let current_commit = match get_current_commit_hash()? {
        Some(hash) => hash,
        None => return Err("Current branch has no commits".into()),
    };
    
    if target_commit == current_commit {
        println!("Already up to date.");
        return Ok(());
    }
    
    let repo_root = get_repo_root()?;
    restore_files_from_commit(&repo_root, &target_commit)?;
    
    let mut index = Index::default();
    add_all_files_to_index(&repo_root, &mut index)?;
    write_index(&index)?;
    
    let merge_message = if no_ff {
        format!("Merge branch '{}' (no-ff)", branch)
    } else {
        format!("Merge branch '{}'", branch)
    };
    
    create_merge_commit(merge_message, current_commit, target_commit)?;
    
    println!("Merged branch '{}' into '{}'", branch, config.current_branch);
    Ok(())
}

fn add_all_files_to_index(repo_root: &Path, index: &mut Index) -> Result<()> {
    use walkdir::WalkDir;
    use std::fs;
    
    for entry in WalkDir::new(repo_root) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && !path.starts_with(repo_root.join(".kvcs")) {
            let content = fs::read(path)?;
            let hash = hash_content(&content);
            let mode = get_file_mode(path);
            
            store_object(&hash, &content)?;
            
            let relative_path = path.strip_prefix(repo_root)?
                .to_string_lossy()
                .replace('\\', "/");
            
            let index_entry = IndexEntry {
                hash,
                mode,
                stage: 0,
            };
            
            index.files.insert(relative_path, index_entry);
        }
    }
    
    Ok(())
}

fn create_merge_commit(message: String, parent1: String, parent2: String) -> Result<()> {
    let index = read_index()?;
    
    let tree_entries: Vec<TreeEntry> = index.files
        .iter()
        .map(|(name, entry)| TreeEntry {
            name: name.clone(),
            hash: entry.hash.clone(),
            is_file: true,
            mode: entry.mode.clone(),
        })
        .collect();
    
    let tree_content = serde_json::to_vec(&tree_entries)?;
    let tree_hash = hash_content(&tree_content);
    store_object(&tree_hash, &tree_content)?;
    
    let config = read_config()?;
    
    let commit = Commit {
        hash: String::new(),
        message: message.clone(),
        author: format!("{} <{}>", config.user_name, config.user_email),
        timestamp: Utc::now(),
        parent: Some(parent1), // In a real implementation, you'd store both parents
        tree: tree_hash,
    };
    
    let commit_content = serde_json::to_vec(&commit)?;
    let commit_hash = hash_content(&commit_content);
    
    let mut final_commit = commit;
    final_commit.hash = commit_hash.clone();
    let final_commit_content = serde_json::to_vec(&final_commit)?;
    
    store_object(&commit_hash, &final_commit_content)?;
    
    let mut config = read_config()?;
    config.branches.insert(config.current_branch.clone(), commit_hash.clone());
    write_config(&config)?;
    
    let empty_index = Index::default();
    write_index(&empty_index)?;
    
    println!("[{} {}] {}", config.current_branch, &commit_hash[..8], message);
    
    Ok(())
}

use super::*;

fn restore_files_from_commit(repo_root: &Path, commit_hash: &str) -> Result<()> {
    use std::fs;
    
    let commit_data = read_object(commit_hash)?;
    let commit: Commit = serde_json::from_slice(&commit_data)?;
    
    let tree_data = read_object(&commit.tree)?;
    let tree_entries: Vec<TreeEntry> = serde_json::from_slice(&tree_data)?;
    
    for entry in tree_entries {
        if entry.is_file {
            let file_path = repo_root.join(&entry.name);
            
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            let file_content = read_object(&entry.hash)?;
            fs::write(file_path, file_content)?;
        }
    }
    
    Ok(())
}