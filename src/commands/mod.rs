use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub mod add;
pub mod branch;
pub mod checkout;
pub mod commit;
pub mod init;
pub mod log;
pub mod status;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Commit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub parent: Option<String>,
    pub tree: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TreeEntry {
    pub name: String,
    pub hash: String,
    pub is_file: bool,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Index {
    pub files: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub current_branch: String,
    pub branches: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        let mut branches = HashMap::new();
        branches.insert("main".to_string(), String::new());
        
        Self {
            current_branch: "main".to_string(),
            branches,
        }
    }
}

pub fn get_repo_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;
    
    loop {
        if current.join(".kvcs").exists() {
            return Ok(current);
        }
        
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            return Err("Not in a KVCS repository".into());
        }
    }
}

pub fn get_kvcs_dir() -> Result<PathBuf> {
    Ok(get_repo_root()?.join(".kvcs"))
}

pub fn hash_content(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

pub fn read_index() -> Result<Index> {
    let kvcs_dir = get_kvcs_dir()?;
    let index_path = kvcs_dir.join("index");
    
    if !index_path.exists() {
        return Ok(Index::default());
    }
    
    let content = fs::read_to_string(index_path)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn write_index(index: &Index) -> Result<()> {
    let kvcs_dir = get_kvcs_dir()?;
    let index_path = kvcs_dir.join("index");
    
    let content = serde_json::to_string_pretty(index)?;
    fs::write(index_path, content)?;
    Ok(())
}

pub fn read_config() -> Result<Config> {
    let kvcs_dir = get_kvcs_dir()?;
    let config_path = kvcs_dir.join("config");
    
    if !config_path.exists() {
        return Ok(Config::default());
    }
    
    let content = fs::read_to_string(config_path)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn write_config(config: &Config) -> Result<()> {
    let kvcs_dir = get_kvcs_dir()?;
    let config_path = kvcs_dir.join("config");
    
    let content = serde_json::to_string_pretty(config)?;
    fs::write(config_path, content)?;
    Ok(())
}

pub fn store_object(hash: &str, content: &[u8]) -> Result<()> {
    let kvcs_dir = get_kvcs_dir()?;
    let objects_dir = kvcs_dir.join("objects");
    let (prefix, suffix) = hash.split_at(2);
    
    let dir_path = objects_dir.join(prefix);
    fs::create_dir_all(&dir_path)?;
    
    let file_path = dir_path.join(suffix);
    fs::write(file_path, content)?;
    Ok(())
}

pub fn read_object(hash: &str) -> Result<Vec<u8>> {
    let kvcs_dir = get_kvcs_dir()?;
    let objects_dir = kvcs_dir.join("objects");
    let (prefix, suffix) = hash.split_at(2);
    
    let file_path = objects_dir.join(prefix).join(suffix);
    Ok(fs::read(file_path)?)
}

pub fn get_current_commit_hash() -> Result<Option<String>> {
    let config = read_config()?;
    let current_branch = &config.current_branch;
    
    match config.branches.get(current_branch) {
        Some(hash) if !hash.is_empty() => Ok(Some(hash.clone())),
        _ => Ok(None),
    }
}