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

