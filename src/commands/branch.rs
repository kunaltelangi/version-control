use super::*;

pub fn list() -> Result<()> {
    let config = read_config()?;
    
    for (branch_name, _) in &config.branches {
        if branch_name == &config.current_branch {
            println!("* \x1b[32m{}\x1b[0m", branch_name);
        } else {
            println!("  {}", branch_name);
        }
    }
    
    Ok(())
}

pub fn create(branch_name: String) -> Result<()> {
    let mut config = read_config()?;
    
    if config.branches.contains_key(&branch_name) {
        return Err(format!("Branch '{}' already exists", branch_name).into());
    }
    
    let current_commit = get_current_commit_hash()?.unwrap_or_default();
    config.branches.insert(branch_name.clone(), current_commit);
    
    write_config(&config)?;
    
    println!("Created branch '{}'", branch_name);
    Ok(())
}