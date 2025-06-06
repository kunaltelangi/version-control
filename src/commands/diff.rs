use super::*;
use std::fs;

pub fn execute(cached: bool, files: Vec<String>) -> Result<()> {
    if cached {
        show_staged_diff(files)
    } else {
        show_working_diff(files)
    }
}

fn show_working_diff(files: Vec<String>) -> Result<()> {
    let repo_root = get_repo_root()?;
    let index = read_index()?;
    
    let files_to_check = if files.is_empty() {
        index.files.keys().cloned().collect()
    } else {
        files
    };
    
    for file_path in files_to_check {
        if let Some(index_entry) = index.files.get(&file_path) {
            let full_path = repo_root.join(&file_path);
            
            if full_path.exists() {
                let current_content = fs::read_to_string(&full_path)?;
                let indexed_content = String::from_utf8(read_object(&index_entry.hash)?)?;
                
                if current_content != indexed_content {
                    println!("diff --kvcs a/{} b/{}", file_path, file_path);
                    show_text_diff(&indexed_content, &current_content);
                    println!();
                }
            }
        }
    }
    
    Ok(())
}

fn show_staged_diff(files: Vec<String>) -> Result<()> {
    let index = read_index()?;
    let current_commit = get_current_commit_hash()?;
    
    if let Some(commit_hash) = current_commit {
        let commit_data = read_object(&commit_hash)?;
        let commit: Commit = serde_json::from_slice(&commit_data)?;
        let tree_data = read_object(&commit.tree)?;
        let tree_entries: Vec<TreeEntry> = serde_json::from_slice(&tree_data)?;
        
        let mut committed_files = HashMap::new();
        for entry in tree_entries {
            committed_files.insert(entry.name, entry.hash);
        }
        
        let files_to_check = if files.is_empty() {
            index.files.keys().cloned().collect()
        } else {
            files
        };
        
        for file_path in files_to_check {
            if let Some(index_entry) = index.files.get(&file_path) {
                let staged_content = String::from_utf8(read_object(&index_entry.hash)?)?;
                
                if let Some(committed_hash) = committed_files.get(&file_path) {
                    let committed_content = String::from_utf8(read_object(committed_hash)?)?;
                    
                    if staged_content != committed_content {
                        println!("diff --kvcs a/{} b/{}", file_path, file_path);
                        show_text_diff(&committed_content, &staged_content);
                        println!();
                    }
                } else {
                    println!("diff --kvcs /dev/null b/{}", file_path);
                    println!("new file mode 644");
                    println!("--- /dev/null");
                    println!("+++ b/{}", file_path);
                    for line in staged_content.lines() {
                        println!("+{}", line);
                    }
                    println!();
                }
            }
        }
    }
    
    Ok(())
}

fn show_text_diff(old: &str, new: &str) {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    
    println!("--- a/file");
    println!("+++ b/file");
    
    let mut i = 0;
    let mut j = 0;
    
    while i < old_lines.len() || j < new_lines.len() {
        if i < old_lines.len() && j < new_lines.len() && old_lines[i] == new_lines[j] {
            i += 1;
            j += 1;
        } else {
            // Find the next matching line
            let mut found_match = false;
            
            // Look ahead to find matching lines
            for look_ahead in 1..=5 {
                if i + look_ahead < old_lines.len() && j < new_lines.len() 
                   && old_lines[i + look_ahead] == new_lines[j] {
                    // Lines were deleted
                    for k in i..i + look_ahead {
                        println!("-{}", old_lines[k]);
                    }
                    i += look_ahead;
                    found_match = true;
                    break;
                } else if i < old_lines.len() && j + look_ahead < new_lines.len() 
                          && old_lines[i] == new_lines[j + look_ahead] {
                    // Lines were added
                    for k in j..j + look_ahead {
                        println!("+{}", new_lines[k]);
                    }
                    j += look_ahead;
                    found_match = true;
                    break;
                }
            }
            
            if !found_match {
                if i < old_lines.len() {
                    println!("-{}", old_lines[i]);
                    i += 1;
                }
                if j < new_lines.len() {
                    println!("+{}", new_lines[j]);
                    j += 1;
                }
            }
        }
    }
}