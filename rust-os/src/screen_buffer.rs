use crate::style;
use engine_io::IoEvent;

pub struct ScreenBuffer {
    visible: Vec<String>,
    viewport_rows: usize,
    viewport_cols: usize,
    prompt_prefix: String,
    input_line: String,
    cursor_x: u16,
    cursor_y: u16,
}

impl ScreenBuffer {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            visible: Vec::new(),
            viewport_cols: cols,
            viewport_rows: rows,
            prompt_prefix: String::new(),
            input_line: String::new(),
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    pub fn set_viewport(&mut self, cols: usize, rows: usize) {
        self.viewport_cols = cols;
        self.viewport_rows = rows;
    }

    pub fn set_prompt(&mut self, prefix: &str) {
        self.prompt_prefix = prefix.to_string();
    }

    pub fn set_input_line(&mut self, val: &str) {
        self.input_line = val.to_string();
        let prompt_len = style::visible_len(&self.prompt_prefix);
        self.cursor_x = (prompt_len + val.len()) as u16;
        if self.viewport_rows > 0 {
            self.cursor_y = (self.visible.len().min(self.viewport_rows.saturating_sub(1))) as u16;
        }
    }

    pub fn append(&mut self, lines: &[String]) {
        for line in lines {
            let wrapped = self.wrap_line(line);
            self.visible.extend(wrapped);
        }
        // Keep last N visible rows
        let max_scroll = self.viewport_rows * 5;
        if self.visible.len() > max_scroll {
            let drop = self.visible.len() - max_scroll;
            self.visible.drain(0..drop);
        }
    }

    pub fn commit_input_line(&mut self) {
        let line = format!("{}{}", self.prompt_prefix, self.input_line);
        let wrapped = self.wrap_line(&line);
        self.visible.extend(wrapped);
        self.input_line.clear();
    }

    pub fn send_frame(&self) -> IoEvent {
        let start = self.visible.len().saturating_sub(self.viewport_rows.saturating_sub(1));
        let mut lines: Vec<String> = self.visible[start..].to_vec();
        // Add prompt line
        lines.push(format!("{}{}", self.prompt_prefix, self.input_line));

        IoEvent::ScreenFull {
            lines,
            cursor_x: self.cursor_x,
            cursor_y: self.cursor_y,
        }
    }

    pub fn clear(&mut self) {
        self.visible.clear();
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    fn wrap_line(&self, line: &str) -> Vec<String> {
        if self.viewport_cols == 0 {
            return vec![line.to_string()];
        }
        let visible = style::strip(line);
        if visible.len() <= self.viewport_cols {
            return vec![line.to_string()];
        }
        // Simple wrap: split at viewport_cols visible chars, preserving markup
        // For simplicity we wrap on visible length boundary
        let mut result = Vec::new();
        let mut buf = String::new();
        let mut buf_visible_len = 0;
        let mut in_tag = false;
        let mut tag_buf = String::new();

        for c in line.chars() {
            if c == '[' {
                in_tag = true;
                tag_buf.push(c);
            } else if c == ']' && in_tag {
                in_tag = false;
                tag_buf.push(c);
                buf.push_str(&tag_buf);
                tag_buf.clear();
            } else if in_tag {
                tag_buf.push(c);
            } else {
                buf.push(c);
                buf_visible_len += 1;
                if buf_visible_len >= self.viewport_cols {
                    result.push(buf.clone());
                    buf.clear();
                    buf_visible_len = 0;
                }
            }
        }
        if !buf.is_empty() || !tag_buf.is_empty() {
            buf.push_str(&tag_buf);
            result.push(buf);
        }
        if result.is_empty() {
            result.push(String::new());
        }
        result
    }
}
