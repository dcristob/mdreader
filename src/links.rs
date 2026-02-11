pub fn open_link(url: &str) -> anyhow::Result<()> {
    if url.starts_with("http://") || url.starts_with("https://") {
        webbrowser::open(url).map_err(|e| anyhow::anyhow!("Failed to open browser: {}", e))?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("Not an external URL: {}", url))
    }
}
