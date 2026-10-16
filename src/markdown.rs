#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineTok<'a> {
    Plain(&'a str),
    Bold(&'a str),
    Italic(&'a str),
    Highlight(&'a str),
    Strike(&'a str),
    Code(&'a str),
}

pub fn parse_inline_tokens<'a>(mut text: &'a str) -> Vec<InlineTok<'a>> {
    let mut tokens = Vec::new();

    while !text.is_empty() {
        let bold_pos = text.find("**");
        let highlight_pos = text.find("==");
        let strike_pos = text.find("~~");
        let code_pos = text.find('`');
        let italic_pos = text.find('*');

        let mut candidates: Vec<(usize, &str)> = Vec::new();
        if let Some(p) = bold_pos {
            candidates.push((p, "**"));
        }
        if let Some(p) = highlight_pos {
            candidates.push((p, "=="));
        }
        if let Some(p) = strike_pos {
            candidates.push((p, "~~"));
        }
        if let Some(p) = code_pos {
            candidates.push((p, "`"));
        }
        if let Some(p) = italic_pos {
            if bold_pos != Some(p) {
                candidates.push((p, "*"));
            }
        }

        candidates.sort_by_key(|&(p, _)| p);

        if let Some(&(first_pos, delim)) = candidates.first() {
            if first_pos > 0 {
                tokens.push(InlineTok::Plain(&text[..first_pos]));
            }
            let after_delim = &text[first_pos + delim.len()..];
            if let Some(end_pos) = after_delim.find(delim) {
                let inner = &after_delim[..end_pos];
                match delim {
                    "**" => tokens.push(InlineTok::Bold(inner)),
                    "==" => tokens.push(InlineTok::Highlight(inner)),
                    "~~" => tokens.push(InlineTok::Strike(inner)),
                    "`" => tokens.push(InlineTok::Code(inner)),
                    "*" => tokens.push(InlineTok::Italic(inner)),
                    _ => tokens.push(InlineTok::Plain(inner)),
                }
                text = &after_delim[end_pos + delim.len()..];
            } else {
                tokens.push(InlineTok::Plain(&text[first_pos..first_pos + delim.len()]));
                text = after_delim;
            }
        } else {
            tokens.push(InlineTok::Plain(text));
            break;
        }
    }

    tokens
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum MarkdownBlock {
    Heading(usize, String),
    Checklist {
        line_idx: usize,
        is_checked: bool,
        text: String,
    },
    Bullet(String),
    Numbered(String, String),
    Quote(String),
    Callout {
        kind: String,
        text: String,
    },
    CodeBlock {
        language: String,
        code: String,
    },
    Divider,
    Paragraph(String),
    Blank,
}

pub fn parse_markdown_blocks(content: &str) -> Vec<MarkdownBlock> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // 1. Multi-line Code Block
        if trimmed.starts_with("```") {
            let lang = trimmed.trim_start_matches('`').trim().to_string();
            let mut code_lines = Vec::new();
            i += 1;
            while i < lines.len() {
                let code_line = lines[i];
                if code_line.trim().starts_with("```") {
                    i += 1;
                    break;
                }
                code_lines.push(code_line);
                i += 1;
            }
            blocks.push(MarkdownBlock::CodeBlock {
                language: if lang.is_empty() {
                    "code".to_string()
                } else {
                    lang
                },
                code: code_lines.join("\n"),
            });
            continue;
        }

        // 2. Blank line
        if trimmed.is_empty() {
            blocks.push(MarkdownBlock::Blank);
            i += 1;
            continue;
        }

        // 3. Horizontal Rule
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            blocks.push(MarkdownBlock::Divider);
            i += 1;
            continue;
        }

        // 4. Headings
        if let Some(h1) = trimmed.strip_prefix("# ") {
            blocks.push(MarkdownBlock::Heading(1, h1.to_string()));
            i += 1;
            continue;
        }
        if let Some(h2) = trimmed.strip_prefix("## ") {
            blocks.push(MarkdownBlock::Heading(2, h2.to_string()));
            i += 1;
            continue;
        }
        if let Some(h3) = trimmed.strip_prefix("### ") {
            blocks.push(MarkdownBlock::Heading(3, h3.to_string()));
            i += 1;
            continue;
        }

        // 5. Checklist Task: - [ ] or - [x] or - [X]
        if trimmed.starts_with("- [ ] ")
            || trimmed.starts_with("- [x] ")
            || trimmed.starts_with("- [X] ")
        {
            let is_checked = trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ");
            let text = trimmed[6..].to_string();
            blocks.push(MarkdownBlock::Checklist {
                line_idx: i,
                is_checked,
                text,
            });
            i += 1;
            continue;
        }

        // 6. Callouts: > [!NOTE], > [!TIP], > [!WARNING], or emojis 💡, ⚠️, ℹ️, 📌
        if trimmed.starts_with("> [!") {
            if let Some(end_bracket) = trimmed.find(']') {
                let kind = trimmed[4..end_bracket].to_string();
                let mut text = trimmed[end_bracket + 1..].trim().to_string();
                i += 1;
                while i < lines.len() && lines[i].trim().starts_with("> ") {
                    let next_trim = lines[i].trim();
                    text.push(' ');
                    text.push_str(&next_trim[2..]);
                    i += 1;
                }
                blocks.push(MarkdownBlock::Callout { kind, text });
                continue;
            }
        }
        if trimmed.starts_with("💡")
            || trimmed.starts_with("⚠️")
            || trimmed.starts_with("ℹ️")
            || trimmed.starts_with("📌")
        {
            let icon_len = trimmed.chars().next().map(|c| c.len_utf8()).unwrap_or(0);
            let text = trimmed[icon_len..].trim().to_string();
            blocks.push(MarkdownBlock::Callout {
                kind: "NOTE".to_string(),
                text,
            });
            i += 1;
            continue;
        }

        // 7. Blockquote
        if let Some(quote_text) = trimmed.strip_prefix("> ") {
            let mut text = quote_text.to_string();
            i += 1;
            while i < lines.len() && lines[i].trim().starts_with("> ") {
                let next_trim = lines[i].trim();
                text.push(' ');
                text.push_str(&next_trim[2..]);
                i += 1;
            }
            blocks.push(MarkdownBlock::Quote(text));
            continue;
        }

        // 8. Bullet List
        if trimmed.starts_with("• ") || trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let text = trimmed[2..].trim().to_string();
            blocks.push(MarkdownBlock::Bullet(text));
            i += 1;
            continue;
        }

        // 9. Numbered List: e.g. "1. " or "2. "
        if let Some(dot_idx) = trimmed.find(". ") {
            if dot_idx > 0 && dot_idx <= 4 && trimmed[..dot_idx].chars().all(|c| c.is_ascii_digit())
            {
                let num = trimmed[..=dot_idx].to_string();
                let text = trimmed[dot_idx + 2..].trim().to_string();
                blocks.push(MarkdownBlock::Numbered(num, text));
                i += 1;
                continue;
            }
        }

        // 10. Normal Paragraph
        blocks.push(MarkdownBlock::Paragraph(trimmed.to_string()));
        i += 1;
    }

    blocks
}
