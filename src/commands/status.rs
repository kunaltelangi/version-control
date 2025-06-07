use super::*;
use std::collections::HashSet;
use std::fs;
use walkdir::WalkDir;

pub fn execute() -> Result<()> {
    let repo_root = get_repo_root()?;
    let config = read_config()?;
    let index = read_index()?;
    
    println!("On branch {}", config.current_branch);
    
    if let Some(current_commit_hash) = get_current_commit_hash()? {
        let commit_data = read_object(&current_commit_hash)?;
        let commit: Commit = serde_json::from_slice(&commit_data)?;
        println!("Current commit: {} ({})", &current_commit_hash[..8], commit.message);
    } else {
        println!("No commits yet");
    }
    
    println!();
    
    // Collect working files
    let mut working_files = HashSet::new();
    for entry in WalkDir::new(&repo_root) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && !path.starts_with(repo_root.join(".kvcs")) {
            let relative_path = path.strip_prefix(&repo_root)?
                .to_string_lossy()
                .replace('\\', "/");
            working_files.insert(relative_path.to_string());
        }
    }
    
    let staged_files: HashSet<String> = index.files.keys().cloned().collect();
    
    let mut modified_files = Vec::new();
    let mut staged_for_commit = Vec::new();
    let mut deleted_files = Vec::new();
    
    // Check staged files
    for (file_path, index_entry) in &index.files {
        let full_path = repo_root.join(file_path);
        
        if full_path.exists() {
            let content = fs::read(&full_path)?;
            let current_hash = hash_content(&content);
            
            if current_hash != index_entry.hash {
                modified_files.push(file_path.clone());
            } else {
                staged_for_commit.push(file_path.clone());
            }
        } else {
            deleted_files.push(file_path.clone());
        }
    }
    
    let untracked_files: Vec<String> = working_files
        .difference(&staged_files)
        .cloned()
        .collect();
    
    // Display results
    if !staged_for_commit.is_empty() {
        println!("Changes to be committed:");
        println!("  (use \"kvcs reset HEAD <file>...\" to unstage)");
        println!();
        for file in &staged_for_commit {
            println!("        \x1b[32mnew file:   {}\x1b[0m", file);
        }
        println!();
    }
    
    if !modified_files.is_empty() {
        println!("Changes not staged for commit:");
        println!("  (use \"kvcs add <file>...\" to update what will be committed)");
        println!("  (use \"kvcs checkout -- <file>...\" to discard changes in working directory)");
        println!();
        for file in &modified_files {
            println!("        \x1b[31mmodified:   {}\x1b[0m", file);
        }
        println!();
    }
    
    if !deleted_files.is_empty() {
        println!("Changes not staged for commit:");
        println!("  (use \"kvcs add <file>...\" to update what will be committed)");
        println!("  (use \"kvcs checkout -- <file>...\" to discard changes in working directory)");
        println!();
        for file in &deleted_files {
            println!("        \x1b[31mdeleted:    {}\x1b[0m", file);
        }
        println!();
    }
    
    if !untracked_files.is_empty() {
        println!("Untracked files:");
        println!("  (use \"kvcs add <file>...\" to include in what will be committed)");
        println!();
        for file in &untracked_files {
            println!("        \x1b[31m{}\x1b[0m", file);
        }
        println!();
    }
    
    if staged_for_commit.is_empty() && modified_files.is_empty() && 
       deleted_files.is_empty() && untracked_files.is_empty() {
        println!("nothing to commit, working tree clean");
    }
    
    Ok(())
}
