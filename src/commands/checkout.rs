use super::*;
use std::fs;

pub fn execute(target: String) -> Result<()> {
    let repo_root = get_repo_root()?;
    let mut config = read_config()?;
    if config.branches.contains_key(&target) {
        config.current_branch = target.clone();
        write_config(&config)?;
        
        if let Some(commit_hash) = config.branches.get(&target) {
            if !commit_hash.is_empty() {
                restore_files_from_commit(&repo_root, commit_hash)?;
            }
        }
        
        println!("Switched to branch '{}'", target);
        Ok(())
    } else {
        if target.len() >= 8 {
            let kvcs_dir = get_kvcs_dir()?;
            let objects_dir = kvcs_dir.join("objects");
            
            if target.len() >= 2 {
                let (prefix, suffix) = target.split_at(2);
                let dir_path = objects_dir.join(prefix);
                
                if dir_path.exists() {
                    for entry in fs::read_dir(dir_path)? {
                        let entry = entry?;
                        let filename = entry.file_name().to_string_lossy().to_string();
                        let full_hash = format!("{}{}", prefix, filename);
                        
                        if full_hash.starts_with(&target) {
                            restore_files_from_commit(&repo_root, &full_hash)?;
                            println!("HEAD is now at {} (detached)", &full_hash[..8]);
                            return Ok(());
                        }
                    }
                }
            }
        }
        
        Err(format!("Branch or commit '{}' not found", target).into())
    }
}

fn restore_files_from_commit(repo_root: &Path, commit_hash: &str) -> Result<()> {
    let commit_data = read_object(commit_hash)?;
    let commit: Commit = serde_json::from_slice(&commit_data)?;
    
    let tree_data = read_object(&commit.tree)?;
    let tree_entries: Vec<TreeEntry> = serde_json::from_slice(&tree_data)?;
    
    clear_working_directory(repo_root)?;
    
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

fn clear_working_directory(repo_root: &Path) -> Result<()> {
    use walkdir::WalkDir;
    
    let mut files_to_remove = Vec::new();
    let mut dirs_to_remove = Vec::new();
    
    for entry in WalkDir::new(repo_root) {
        let entry = entry?;
        let path = entry.path();
        
        if path.starts_with(repo_root.join(".kvcs")) {
            continue;
        }
        
        if path == repo_root {
            continue;
        }
        
        if path.is_file() {
            files_to_remove.push(path.to_path_buf());
        } else if path.is_dir() {
            dirs_to_remove.push(path.to_path_buf());
        }
    }
    
    // Remove files first
    for file in files_to_remove {
        let _ = fs::remove_file(file);
    }
    dirs_to_remove.sort();
    dirs_to_remove.reverse();
    for dir in dirs_to_remove {
        let _ = fs::remove_dir(dir);
    }
    
    Ok(())
}