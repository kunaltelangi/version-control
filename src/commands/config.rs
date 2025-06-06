use super::*;

pub fn set(key: String, value: String) -> Result<()> {
    let mut config = read_config()?;
    
    match key.as_str() {
        "user.name" => config.user_name = value,
        "user.email" => config.user_email = value,
        _ => return Err(format!("Unknown configuration key: {}", key).into()),
    }
    
    write_config(&config)?;
    println!("Set {} = {}", key, value);
    
    Ok(())
}

pub fn get(key: String) -> Result<()> {
    let config = read_config()?;
    
    let value = match key.as_str() {
        "user.name" => &config.user_name,
        "user.email" => &config.user_email,
        _ => return Err(format!("Unknown configuration key: {}", key).into()),
    };
    
    println!("{}", value);
    Ok(())
}

pub fn list() -> Result<()> {
    let config = read_config()?;
    
    println!("user.name={}", config.user_name);
    println!("user.email={}", config.user_email);
    
    Ok(())
}