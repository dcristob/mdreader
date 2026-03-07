#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub matches: Vec<Match>,
    pub current_match: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub start: usize,
    pub end: usize,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            query: String::new(),
            matches: Vec::new(),
            current_match: None,
        }
    }
}

impl SearchState {
    pub fn search(&mut self, content: &str) {
        self.matches.clear();
        self.current_match = None;

        if self.query.is_empty() {
            return;
        }

        let query_lower = self.query.to_lowercase();
        let content_lower = content.to_lowercase();

        let mut start = 0;
        while let Some(pos) = content_lower[start..].find(&query_lower) {
            let match_start = start + pos;
            let match_end = match_start + self.query.len();

            self.matches.push(Match {
                start: match_start,
                end: match_end,
            });
            start = match_end;
        }

        if !self.matches.is_empty() {
            self.current_match = Some(0);
        }
    }

    pub fn next_match(&mut self) {
        if let Some(current) = self.current_match {
            self.current_match = Some((current + 1) % self.matches.len());
        }
    }

    pub fn prev_match(&mut self) {
        if let Some(current) = self.current_match {
            if current == 0 {
                self.current_match = Some(self.matches.len() - 1);
            } else {
                self.current_match = Some(current - 1);
            }
        }
    }

    pub fn has_matches(&self) -> bool {
        !self.matches.is_empty()
    }

    pub fn match_count(&self) -> String {
        if self.matches.is_empty() {
            "No matches".to_string()
        } else {
            format!(
                "{} of {}",
                self.current_match.map(|i| i + 1).unwrap_or(0),
                self.matches.len()
            )
        }
    }
}
