use std::path::PathBuf;
use anyhow::Result;

pub fn load_file(path: &PathBuf) -> Result<String> {
    std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))
}

pub fn load_file_async(path: PathBuf) -> tokio::sync::oneshot::Receiver<Result<String>> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    
    tokio::spawn(async move {
        let result = load_file(&path);
        let _ = tx.send(result);
    });
    
    rx
}
