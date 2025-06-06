use super::*;

pub fn execute(limit: usize, oneline: bool) -> Result<()> {
    let current_commit_hash = match get_current_commit_hash()? {
        Some(hash) => hash,
        None => {
            println!("No commits found");
            return Ok(());
        }
    };
    
    let mut commit_hash = Some(current_commit_hash);
    let mut count = 0;
    
    while let Some(hash) = commit_hash {
        if count >= limit {
            break;
        }
        
        let commit_data = read_object(&hash)?;
        let commit: Commit = serde_json::from_slice(&commit_data)?;
        
        if oneline {
            println!("{} {}", &hash[..7], commit.message);
        } else {
            println!("\x1b[33mcommit {}\x1b[0m", hash);
            println!("Author: {}", commit.author);
            println!("Date: {}", commit.timestamp.format("%a %b %d %H:%M:%S %Y %z"));
            println!();
            println!("    {}", commit.message);
            println!();
        }
        
        commit_hash = commit.parent;
        count += 1;
    }
    
    Ok(())
}