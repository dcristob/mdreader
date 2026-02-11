use std::process::Command;

pub fn open_external_link(url: &str) -> anyhow::Result<()> {
    if url.starts_with("http://") || url.starts_with("https://") {
        let result = Command::new("xdg-open").arg(url).spawn();

        if result.is_ok() {
            return Ok(());
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(url)
                .spawn()
                .map_err(|e| anyhow::anyhow!("Failed to open link: {}", e))?;
        }

        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(["/C", "start", url])
                .spawn()
                .map_err(|e| anyhow::anyhow!("Failed to open link: {}", e))?;
        }

        return Ok(());
    }

    Err(anyhow::anyhow!("Not an external URL: {}", url))
}
