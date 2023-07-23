pub(crate) struct Buffer {
    /// Buffer to store the set of lines which correspond to a full plist message from
    /// powermetrics. A plist message may contain up to ~2000 lines (depending on the chip).
    buffer: Vec<String>,
}

impl Buffer {
    pub(crate) fn new() -> Self {
        Self {
            buffer: Vec::<String>::new(),
        }
    }

    pub(crate) fn append_line(&mut self, line: String) {
        // Each line but the last is appended to the buffer.
        if line.starts_with(char::from(0)) {
            // Trim the leading null character if present (happens only on the 1st line).
            let line = line.trim_start_matches(char::from(0)).to_string();
            self.buffer.push(line);
        } else {
            self.buffer.push(line);
        }
    }

    pub(crate) fn append_last_line(&mut self, line: String) {
        // When the last line of the message is reached, clean-up invalid fields and create the
        // whole plist buffer for parsing.
        self.buffer.push(line);

        // Fix a powermetrics bug by removing the last `idle_ratio` line. This should be the
        // (n-5)th line, so we only iterate over the last 10 lines.
        let pos = self
            .buffer
            .iter()
            .rev()
            .take(10)
            .position(|line| line.starts_with("<key>idle_ratio</key>"));
        if let Some(pos) = pos {
            self.buffer.remove(self.buffer.len() - pos - 1);
        }
    }

    /// Create the final plist message and clear the buffer.
    pub(crate) fn finalize(&mut self) -> String {
        let plist = self.buffer.join("\n");
        self.buffer.clear();
        plist
    }
}
