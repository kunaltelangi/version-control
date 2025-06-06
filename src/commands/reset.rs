use super::*;
use std::fs;
pub fn execute(hard: bool, soft: bool, commit: Option<String>) -> Result<()> {
    if hard && soft {
        return Err("Cannot use both --hard and --soft flags".into());
    }
    let target_commit = match commit {
        Some(commit_hash) => {
            if read_object(&commit_hash).is_err() {
                return Err(format!("Commit '{}' not found", commit_hash).into());
            }
            commit_hash
        }
        None => {
            match get_current_commit_hash()? {
                Some(hash) => hash,
                None => return Err("No commits to reset to".into()),
            }
        }
    };

    if hard {
        reset_hard(&target_commit)?;
    } else if soft {
        reset_soft(&target_commit)?;
    } else {
        reset_mixed(&target_commit)?;
    }

    println!("HEAD is now at {}", &target_commit[..8]);
    Ok(())
}
fn reset_hard(target_commit: &str) -> Result<()> {
    let mut config = read_config()?;
    let current_branch = config.current_branch.clone();
    config
        .branches
        .insert(current_branch.clone(), target_commit.to_string());
    write_config(&config)?;

    let repo_root = get_repo_root()?;
    clear_working_directory(&repo_root)?;
    restore_files_from_commit(&repo_root, target_commit)?;

    write_index_from_commit(target_commit)?;

    Ok(())
}

fn reset_soft(target_commit: &str) -> Result<()> {
    let mut config = read_config()?;
    let current_branch = config.current_branch.clone();

    config
        .branches
        .insert(current_branch.clone(), target_commit.to_string());
    write_config(&config)?;
    Ok(())
}


fn reset_mixed(target_commit: &str) -> Result<()> {
    let mut config = read_config()?;
    let current_branch = config.current_branch.clone();
    config
        .branches
        .insert(current_branch.clone(), target_commit.to_string());
    write_config(&config)?;

    write_index_from_commit(target_commit)?;
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

fn write_index_from_commit(target_commit: &str) -> Result<()> {
    let commit_data = read_object(target_commit)?;
    let commit: Commit = serde_json::from_slice(&commit_data)?;
    let tree_data = read_object(&commit.tree)?;
    let tree_entries: Vec<TreeEntry> = serde_json::from_slice(&tree_data)?;

    let mut new_index = Index::default();
    for entry in tree_entries {
        let index_entry = IndexEntry {
            hash: entry.hash.clone(),
            mode: entry.mode.clone(),
            stage: 0,
        };
        new_index.files.insert(entry.name.clone(), index_entry);
    }

    // Write to disk
    write_index(&new_index)?;
    Ok(())
}
