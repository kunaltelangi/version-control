use super::*;
use std::fs;
use walkdir::WalkDir;

pub fn execute(files: Vec<String>) -> Result<()> {
    let repo_root = get_repo_root()?;
    let mut index = read_index()?;
    
    if files.is_empty() {
        return Err("No files specified".into());
    }
    
    for file_pattern in files {
        if file_pattern == "." {
            add_directory(&repo_root, &mut index)?;
        } else {
            let file_path = repo_root.join(&file_pattern);
            
            if file_path.is_file() {
                add_file(&repo_root, &file_path, &mut index)?;
            } else if file_path.is_dir() {
                add_directory(&file_path, &mut index)?;
            } else {
                let mut found = false;
                for entry in WalkDir::new(&repo_root) {
                    let entry = entry?;
                    let path = entry.path();
                    
                    if path.is_file() {
                        let relative_path = path.strip_prefix(&repo_root)?;
                        if relative_path.to_string_lossy().contains(&file_pattern) {
                            add_file(&repo_root, path, &mut index)?;
                            found = true;
                        }
                    }
                }
                
                if !found {
                    println!("Warning: '{}' did not match any files", file_pattern);
                }
            }
        }
    }
    
    write_index(&index)?;
    Ok(())
}

fn add_file(repo_root: &Path, file_path: &Path, index: &mut Index) -> Result<()> {
    if file_path.starts_with(repo_root.join(".kvcs")) {
        return Ok(());
    }
    
    let content = fs::read(file_path)?;
    let hash = hash_content(&content);
    
    store_object(&hash, &content)?;
    
    let relative_path = file_path.strip_prefix(repo_root)?
        .to_string_lossy()
        .replace('\\', "/"); 
    
    index.files.insert(relative_path.to_string(), hash);
    println!("Added '{}'", relative_path);
    
    Ok(())
}

fn add_directory(dir_path: &Path, index: &mut Index) -> Result<()> {
    let repo_root = get_repo_root()?;
    
    for entry in WalkDir::new(dir_path) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            add_file(&repo_root, path, index)?;
        }
    }
    
    Ok(())
}