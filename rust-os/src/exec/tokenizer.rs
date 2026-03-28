/// Shell tokenizer: handles pipes, redirects, semicolons, quoting.

#[derive(Debug, Clone)]
pub struct SimpleCommand {
    pub tokens: Vec<String>,
    pub redirect_file: Option<String>,
    pub redirect_append: bool,
}

/// Parse a shell input line into a list of SimpleCommands (separated by ';').
/// Pipe ('|') is handled by chaining commands, but for simplicity we flatten
/// pipes into sequential execution (stdout capture not yet needed for this game).
pub fn tokenize(input: &str) -> Vec<SimpleCommand> {
    let mut commands = Vec::new();
    // Split by ';' first
    for segment in split_semicolons(input) {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        commands.push(parse_simple(segment));
    }
    commands
}

fn split_semicolons(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = ' ';
    for c in s.chars() {
        if in_quote {
            if c == quote_char {
                in_quote = false;
            } else {
                current.push(c);
            }
        } else if c == '"' || c == '\'' {
            in_quote = true;
            quote_char = c;
        } else if c == ';' {
            result.push(current.clone());
            current.clear();
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

fn parse_simple(input: &str) -> SimpleCommand {
    let mut tokens = Vec::new();
    let mut redirect_file = None;
    let mut redirect_append = false;
    let parts = tokenize_words(input);

    let mut i = 0;
    while i < parts.len() {
        if parts[i] == ">>" {
            redirect_append = true;
            if i + 1 < parts.len() {
                redirect_file = Some(parts[i + 1].clone());
                i += 2;
            } else {
                i += 1;
            }
        } else if parts[i] == ">" {
            redirect_append = false;
            if i + 1 < parts.len() {
                redirect_file = Some(parts[i + 1].clone());
                i += 2;
            } else {
                i += 1;
            }
        } else if parts[i] == "|" {
            // For now, skip pipe support — treat rest as separate command
            i += 1;
        } else {
            tokens.push(parts[i].clone());
            i += 1;
        }
    }

    SimpleCommand {
        tokens,
        redirect_file,
        redirect_append,
    }
}

fn tokenize_words(input: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = ' ';
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quote {
            if c == quote_char {
                in_quote = false;
            } else {
                current.push(c);
            }
        } else if c == '"' || c == '\'' {
            in_quote = true;
            quote_char = c;
        } else if c == '>' {
            if !current.is_empty() {
                words.push(current.clone());
                current.clear();
            }
            if chars.peek() == Some(&'>') {
                chars.next();
                words.push(">>".to_string());
            } else {
                words.push(">".to_string());
            }
        } else if c == '|' {
            if !current.is_empty() {
                words.push(current.clone());
                current.clear();
            }
            words.push("|".to_string());
        } else if c.is_whitespace() {
            if !current.is_empty() {
                words.push(current.clone());
                current.clear();
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}
