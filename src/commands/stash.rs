use super::*;
use chrono::Utc;

pub fn push(message: Option<String>) -> Result<()> {
    let repo_root = get_repo_root()?;
    let index = read_index()?;
    let config = read_config()?;
    let mut stash = read_stash()?;
    
    if index.files.is_empty() {
        return Err("No changes to stash".into());
    }
    
    let current_commit = get_current_commit_hash()?.unwrap_or_default();
    let stash_message = message.unwrap_or_else(|| 
        format!("WIP on {}: {}", config.current_branch, 
                if current_commit.is_empty() { "initial commit".to_string() } 
                else { current_commit[..8].to_string() }));
    
    let stash_entry = StashEntry {
        message: stash_message.clone(),
        branch: config.current_branch.clone(),
        commit_hash: current_commit,
        index_state: index.clone(),
        timestamp: Utc::now(),
    };
    
    stash.entries.insert(0, stash_entry);
    write_stash(&stash)?;
    
    // Clear the index
    let empty_index = Index::default();
    write_index(&empty_index)?;
    
    println!("Saved working directory and index state on {}: {}", 
             config.current_branch, stash_message);
    
    Ok(())
}

pub fn pop() -> Result<()> {
    let mut stash = read_stash()?;
    
    if stash.entries.is_empty() {
        return Err("No stash entries found".into());
    }
    
    let stash_entry = stash.entries.remove(0);
    write_stash(&stash)?;
    
    // Restore the index
    write_index(&stash_entry.index_state)?;
    
    println!("Dropped refs/stash@{{0}} ({})", stash_entry.message);
    
    Ok(())
}

pub fn list() -> Result<()> {
    let stash = read_stash()?;
    
    if stash.entries.is_empty() {
        println!("No stash entries");
        return Ok(());
    }
    
    for (i, entry) in stash.entries.iter().enumerate() {
        println!("stash@{{{}}}: On {}: {}", i, entry.branch, entry.message);
    }
    
    Ok(())
}

pub fn show(index: Option<usize>) -> Result<()> {
    let stash = read_stash()?;
    let idx = index.unwrap_or(0);
    
    if idx >= stash.entries.len() {
        return Err("Invalid stash index".into());
    }
    
    let entry = &stash.entries[idx];
    println!("stash@{{{}}}: {}", idx, entry.message);
    println!("Date: {}", entry.timestamp.format("%a %b %d %H:%M:%S %Y %z"));
    println!();
    
    for (file, index_entry) in &entry.index_state.files {
        println!("    {}", file);
    }
    
    Ok(())
}

pub fn drop(index: usize) -> Result<()> {
    let mut stash = read_stash()?;
    
    if index >= stash.entries.len() {
        return Err("Invalid stash index".into());
    }
    
    let removed = stash.entries.remove(index);
    write_stash(&stash)?;
    
    println!("Dropped stash@{{{}}} ({})", index, removed.message);
    
    Ok(())
}