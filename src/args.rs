use std::path::PathBuf;

pub struct Args {
    pub file: Option<PathBuf>,
}

impl Args {
    pub fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let file = args.get(1).map(PathBuf::from);
        Self { file }
    }
}
