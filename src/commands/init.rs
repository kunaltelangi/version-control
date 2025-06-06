use super::*;
use std::fs;

pub fn execute() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let kvcs_dir = current_dir.join(".vcs");
    
    if kvcs_dir.exists() {
        return Err("Repository already initialized".into());
    }
    
    fs::create_dir_all(&kvcs_dir)?;
    fs::create_dir_all(kvcs_dir.join("objects"))?;
    fs::create_dir_all(kvcs_dir.join("refs").join("heads"))?;
    
    let config = Config::default();
    write_config(&config)?;
    
    let index = Index::default();
    write_index(&index)?;
    
    let head_path = kvcs_dir.join("HEAD");
    fs::write(head_path, "ref: refs/heads/main\n")?;
    
    println!("Initialized empty KVCS repository in {}", kvcs_dir.display());
    Ok(())
}