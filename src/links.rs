use std::path::{Path, PathBuf};

pub enum LinkType {
    External(String),
    Internal(PathBuf),
}

pub fn classify_link(url: &str, base_path: Option<&Path>) -> LinkType {
    if url.starts_with("http://") || url.starts_with("https://") {
        LinkType::External(url.to_string())
    } else {
        let path = if let Some(base) = base_path {
            if let Some(parent) = base.parent() {
                parent.join(url)
            } else {
                PathBuf::from(url)
            }
        } else {
            PathBuf::from(url)
        };
        LinkType::Internal(path)
    }
}

pub fn open_link(url: &str) -> anyhow::Result<()> {
    if url.starts_with("http://") || url.starts_with("https://") {
        webbrowser::open(url).map_err(|e| anyhow::anyhow!("Failed to open browser: {}", e))?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("Not an external URL: {}", url))
    }
}
