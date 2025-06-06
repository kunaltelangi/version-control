use super::*;
use chrono::Utc;

pub fn execute(message: String, all: bool) -> Result<()> {
    let repo_root = get_repo_root()?;
    let mut index = read_index()?;

    // If -a flag is used, add all tracked files that have been modified
    if all {
        add_all_modified_files(&repo_root, &mut index)?;
        write_index(&index)?;
    }
    
    if index.files.is_empty() {
        return Err("No changes added to commit".into());
    }
    
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
    
    let parent = get_current_commit_hash()?;
    let config = read_config()?;
    
    let commit = Commit {
        hash: String::new(),
        message: message.clone(),
        author: format!("{} <{}>", config.user_name, config.user_email),
        timestamp: Utc::now(),
        parent,
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
    
    println!("[{} {}] {}", 
             config.current_branch, 
             &commit_hash[..8], 
             message);
    
    println!("{} files changed", index.files.len());
    
    Ok(())
}

fn add_all_modified_files(repo_root: &Path, index: &mut Index) -> Result<()> {
    use walkdir::WalkDir;
    use std::fs;
    
    for entry in WalkDir::new(repo_root) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && !path.starts_with(repo_root.join(".kvcs")) {
            let relative_path = path.strip_prefix(repo_root)?
                .to_string_lossy()
                .replace('\\', "/");
            
            // Check if file exists in index or is being tracked
            if index.files.contains_key(&relative_path) {
                let content = fs::read(path)?;
                let hash = hash_content(&content);
                let mode = get_file_mode(path);
                
                store_object(&hash, &content)?;
                
                let index_entry = IndexEntry {
                    hash,
                    mode,
                    stage: 0,
                };
                
                index.files.insert(relative_path, index_entry);
            }
        }
    }
    
    Ok(())
}
