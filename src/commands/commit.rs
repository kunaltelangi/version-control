use super::*;
use chrono::Utc;

pub fn execute(message: String) -> Result<()> {
    let index = read_index()?;
    
    if index.files.is_empty() {
        return Err("No changes added to commit".into());
    }
    
    let tree_entries: Vec<TreeEntry> = index.files
        .iter()
        .map(|(name, hash)| TreeEntry {
            name: name.clone(),
            hash: hash.clone(),
            is_file: true,
        })
        .collect();
    
    let tree_content = serde_json::to_vec(&tree_entries)?;
    let tree_hash = hash_content(&tree_content);
    
    store_object(&tree_hash, &tree_content)?;
    
    // Get parent commit
    let parent = get_current_commit_hash()?;
    
    let commit = Commit {
        hash: String::new(), 
        message,
        author: "User <user@example.com>".to_string(), // TODO: Make configurable
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
             final_commit.message);
    
    println!("{} files changed", index.files.len());
    
    Ok(())
}