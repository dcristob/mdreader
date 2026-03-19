use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    pub receiver: Receiver<notify::Result<notify::Event>>,
}

impl FileWatcher {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let (tx, rx) = channel();
        
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default(),
        )?;
        
        watcher.watch(path, RecursiveMode::NonRecursive)?;
        
        Ok(Self {
            _watcher: watcher,
            receiver: rx,
        })
    }
}

use anyhow::Result;

pub fn load_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))
}

pub fn strip_mdx_imports(content: &str) -> String {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !trimmed.starts_with("import ") && !trimmed.starts_with("export ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_mdx_imports_removes_import_lines() {
        let input = "import { Callout } from '@components'\nimport React from 'react'\n\n# Hello\n\nSome text";
        let result = strip_mdx_imports(input);
        assert_eq!(result, "\n# Hello\n\nSome text");
    }

    #[test]
    fn test_strip_mdx_imports_removes_export_lines() {
        let input = "export const meta = { title: 'Test' }\n\n# Hello";
        let result = strip_mdx_imports(input);
        assert_eq!(result, "\n# Hello");
    }

    #[test]
    fn test_strip_mdx_imports_preserves_normal_markdown() {
        let input = "# Hello\n\nSome text with **bold**\n\n- list item";
        let result = strip_mdx_imports(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_strip_mdx_imports_handles_leading_whitespace() {
        let input = "  import Something from 'somewhere'\n\n# Title";
        let result = strip_mdx_imports(input);
        assert_eq!(result, "\n# Title");
    }

    #[test]
    fn test_strip_mdx_imports_preserves_prose_with_import_word() {
        // "import" mid-sentence should NOT be stripped
        let input = "# Title\n\nYou can import data from the API.";
        let result = strip_mdx_imports(input);
        assert_eq!(result, input);
    }
}
