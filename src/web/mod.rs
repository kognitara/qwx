//! # Web Reader Module for QWX
//!
//! Provides a text-mode web browser and reader interface.
//!
//! Features:
//! - Content extraction and HTML cleaning (Reader Mode).
//! - Clean rendering with terminal styling (headings, code blocks, blockquotes, lists).
//! - Hyperlink numbering, extraction, cycling, and direct jump (`[1]`, `[2]`, ...).
//! - Browsing history with Back/Forward navigation stack.
//! - Search within page with match jumping.
//! - Bookmarks management.
//! - View modes: Reader Mode, Links List Mode, Raw Source Mode.
//! - Direct URL bar and smart search query resolution (DuckDuckGo, Wikipedia, GitHub, Crates.io).

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::time::Instant;
use unicode_width::UnicodeWidthStr;

/// Style applied to a span of text in the rendered web page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStyle {
    Normal,
    Bold,
    Italic,
    Code,
    Link { id: usize, url: String },
    Header(u8),
    Muted,
    Accent,
    Success,
    Warning,
    Error,
}

/// A formatted slice of text with a specific style.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebLineSpan {
    pub text: String,
    pub style: SpanStyle,
}

impl WebLineSpan {
    pub fn new(text: impl Into<String>, style: SpanStyle) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    pub fn normal(text: impl Into<String>) -> Self {
        Self::new(text, SpanStyle::Normal)
    }

    pub fn link(text: impl Into<String>, id: usize, url: impl Into<String>) -> Self {
        Self::new(text, SpanStyle::Link { id, url: url.into() })
    }

    pub fn code(text: impl Into<String>) -> Self {
        Self::new(text, SpanStyle::Code)
    }

    pub fn bold(text: impl Into<String>) -> Self {
        Self::new(text, SpanStyle::Bold)
    }

    pub fn header(text: impl Into<String>, level: u8) -> Self {
        Self::new(text, SpanStyle::Header(level))
    }

    pub fn muted(text: impl Into<String>) -> Self {
        Self::new(text, SpanStyle::Muted)
    }

    pub fn accent(text: impl Into<String>) -> Self {
        Self::new(text, SpanStyle::Accent)
    }
}

/// Type of line for layout and vertical spacing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineType {
    Empty,
    Title,
    Header(u8),
    Paragraph,
    ListItem,
    Blockquote,
    CodeBlock,
    HorizontalRule,
    Status,
}

/// A rendered line of web content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebLine {
    pub line_type: LineType,
    pub spans: Vec<WebLineSpan>,
}

impl WebLine {
    pub fn new(line_type: LineType, spans: Vec<WebLineSpan>) -> Self {
        Self { line_type, spans }
    }

    pub fn empty() -> Self {
        Self {
            line_type: LineType::Empty,
            spans: Vec::new(),
        }
    }

    pub fn from_spans(spans: Vec<WebLineSpan>) -> Self {
        Self {
            line_type: LineType::Paragraph,
            spans,
        }
    }

    pub fn raw_text(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.spans.is_empty() || self.spans.iter().all(|s| s.text.trim().is_empty())
    }
}

/// An interactive hyperlink extracted from the web document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebLink {
    pub id: usize,
    pub text: String,
    pub url: String,
    pub line_idx: usize,
}

/// Represents a parsed and formatted Web Page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPage {
    pub url: String,
    pub title: String,
    pub lines: Vec<WebLine>,
    pub links: Vec<WebLink>,
    pub raw_html: Option<String>,
    pub status_code: u16,
    pub content_type: String,
    pub fetch_duration_ms: u128,
}

impl WebPage {
    pub fn new(url: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            title: title.into(),
            lines: Vec::new(),
            links: Vec::new(),
            raw_html: None,
            status_code: 200,
            content_type: "text/html".to_string(),
            fetch_duration_ms: 0,
        }
    }

    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    pub fn find_link_by_id(&self, id: usize) -> Option<&WebLink> {
        self.links.iter().find(|l| l.id == id)
    }

    /// Searches for query substring in the page lines, returning matching line indices.
    pub fn search_text(&self, query: &str) -> Vec<usize> {
        if query.trim().is_empty() {
            return Vec::new();
        }
        let q = query.to_lowercase();
        self.lines
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                if line.raw_text().to_lowercase().contains(&q) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Converts a list of search results from the `search` module into a structured, readable WebPage.
    pub fn from_search_results(
        query: &str,
        provider_name: &str,
        results: &[crate::search::SearchResultItem],
        _wrap_width: usize,
    ) -> Self {
        let mut lines = Vec::new();
        let mut links = Vec::new();
        let mut link_id = 1;

        // Title Line
        lines.push(WebLine::new(
            LineType::Title,
            vec![
                WebLineSpan::accent("🔍 Search Results: "),
                WebLineSpan::bold(query),
                WebLineSpan::muted(format!(" [{}]", provider_name)),
            ],
        ));
        lines.push(WebLine::empty());

        if results.is_empty() {
            lines.push(WebLine::new(
                LineType::Paragraph,
                vec![WebLineSpan::muted(format!(
                    "No results found for '{}' under {}.",
                    query, provider_name
                ))],
            ));
        } else {
            lines.push(WebLine::new(
                LineType::Status,
                vec![WebLineSpan::muted(format!(
                    "Showing {} result(s):",
                    results.len()
                ))],
            ));
            lines.push(WebLine::empty());

            for item in results {
                let current_line_idx = lines.len();

                // Item Title with numbered Link
                let mut title_spans = Vec::new();
                title_spans.push(WebLineSpan::normal("• "));

                if !item.url.is_empty() {
                    let link_text = format!("[{}] {}", link_id, item.title);
                    title_spans.push(WebLineSpan::link(link_text, link_id, &item.url));
                    links.push(WebLink {
                        id: link_id,
                        text: item.title.clone(),
                        url: item.url.clone(),
                        line_idx: current_line_idx,
                    });
                    link_id += 1;
                } else {
                    title_spans.push(WebLineSpan::bold(&item.title));
                }

                if !item.extra_info.is_empty() {
                    title_spans.push(WebLineSpan::normal(" "));
                    title_spans.push(WebLineSpan::accent(format!("({})", item.extra_info)));
                }

                lines.push(WebLine::new(LineType::Header(3), title_spans));

                // Description
                if !item.description.is_empty() {
                    let clean_desc = HtmlReaderEngine::clean_text(&item.description);
                    for desc_line in clean_desc.lines() {
                        let trimmed = desc_line.trim();
                        if !trimmed.is_empty() {
                            lines.push(WebLine::new(
                                LineType::Paragraph,
                                vec![
                                    WebLineSpan::normal("    "),
                                    WebLineSpan::muted(trimmed),
                                ],
                            ));
                        }
                    }
                }

                // URL info if available
                if !item.url.is_empty() {
                    lines.push(WebLine::new(
                        LineType::ListItem,
                        vec![
                            WebLineSpan::normal("    URL: "),
                            WebLineSpan::muted(&item.url),
                        ],
                    ));
                }

                lines.push(WebLine::empty());
            }
        }

        Self {
            url: format!("search://{}/?q={}", provider_name.to_lowercase(), query),
            title: format!("Search: {} ({})", query, provider_name),
            lines,
            links,
            raw_html: None,
            status_code: 200,
            content_type: "text/search-results".to_string(),
            fetch_duration_ms: 0,
        }
    }
}

/// Bookmark stored by the user.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebBookmark {
    pub title: String,
    pub url: String,
    pub tags: Vec<String>,
}

/// Navigation history stack.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WebHistory {
    pub entries: Vec<String>,
    pub current_idx: usize,
}

impl WebHistory {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            current_idx: 0,
        }
    }

    pub fn push(&mut self, url: impl Into<String>) {
        let url_str = url.into();
        if self.entries.is_empty() {
            self.entries.push(url_str);
            self.current_idx = 0;
            return;
        }

        // Truncate any forward history if we navigated to a new page
        if self.current_idx + 1 < self.entries.len() {
            self.entries.truncate(self.current_idx + 1);
        }

        // Avoid pushing identical consecutive URL
        if self.entries.last().map(|s| s.as_str()) != Some(&url_str) {
            self.entries.push(url_str);
            self.current_idx = self.entries.len() - 1;
        }
    }

    pub fn can_go_back(&self) -> bool {
        self.current_idx > 0
    }

    pub fn can_go_forward(&self) -> bool {
        !self.entries.is_empty() && self.current_idx + 1 < self.entries.len()
    }

    pub fn back(&mut self) -> Option<&String> {
        if self.can_go_back() {
            self.current_idx -= 1;
            self.entries.get(self.current_idx)
        } else {
            None
        }
    }

    pub fn forward(&mut self) -> Option<&String> {
        if self.can_go_forward() {
            self.current_idx += 1;
            self.entries.get(self.current_idx)
        } else {
            None
        }
    }

    pub fn current(&self) -> Option<&String> {
        self.entries.get(self.current_idx)
    }
}

/// Modes for displaying web content in the browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebReaderViewMode {
    Reader,
    LinksList,
    RawSource,
}

impl WebReaderViewMode {
    pub fn name(&self) -> &'static str {
        match self {
            WebReaderViewMode::Reader => "Reader Mode",
            WebReaderViewMode::LinksList => "Links Index",
            WebReaderViewMode::RawSource => "Raw HTML",
        }
    }
}

/// URL resolution and search query shortcuts.
pub struct UrlHelper;

impl UrlHelper {
    /// Detects if input is a search provider shortcut (e.g. `gh:`, `wiki:`, `cve:`, `hn:`, `ddg:`).
    pub fn search_provider_for_query(input: &str) -> Option<(crate::search::SearchProvider, &str)> {
        let trimmed = input.trim();
        if let Some(rest) = trimmed.strip_prefix("gh:") {
            Some((crate::search::SearchProvider::GitHub, rest.trim()))
        } else if let Some(rest) = trimmed.strip_prefix("gitlab:") {
            Some((crate::search::SearchProvider::GitLab, rest.trim()))
        } else if let Some(rest) = trimmed.strip_prefix("wiki:") {
            Some((crate::search::SearchProvider::Wikipedia, rest.trim()))
        } else if let Some(rest) = trimmed.strip_prefix("cve:") {
            Some((crate::search::SearchProvider::Cve, rest.trim()))
        } else if let Some(rest) = trimmed.strip_prefix("hn:") {
            Some((crate::search::SearchProvider::HackerNews, rest.trim()))
        } else if let Some(rest) = trimmed.strip_prefix("audit:") {
            Some((crate::search::SearchProvider::LocalAudit, rest.trim()))
        } else if let Some(rest) = trimmed.strip_prefix("web:") {
            Some((crate::search::SearchProvider::Web, rest.trim()))
        } else if let Some(rest) = trimmed.strip_prefix("ddg:") {
            Some((crate::search::SearchProvider::Web, rest.trim()))
        } else {
            None
        }
    }

    /// Resolves user input into a valid HTTP/HTTPS URL.
    /// Supports shortcuts:
    /// - `ddg:query` -> DuckDuckGo HTML
    /// - `wiki:query` -> Wikipedia English search
    /// - `crates:query` -> Crates.io search
    /// - `gh:query` -> GitHub search
    /// - `cve:query` -> NVD CVE search
    /// - `hn:query` -> HackerNews Algolia search
    /// - `gitlab:query` -> GitLab project search
    /// - Plain text / query without dots/scheme -> DuckDuckGo search
    pub fn resolve(input: &str) -> String {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return "https://html.duckduckgo.com/html/".to_string();
        }

        if let Some(query) = trimmed.strip_prefix("ddg:").or_else(|| trimmed.strip_prefix("web:")) {
            return format!(
                "https://html.duckduckgo.com/html/?q={}",
                urlencoding::encode(query.trim())
            );
        }
        if let Some(query) = trimmed.strip_prefix("wiki:") {
            return format!(
                "https://en.wikipedia.org/wiki/Special:Search?search={}",
                urlencoding::encode(query.trim())
            );
        }
        if let Some(query) = trimmed.strip_prefix("crates:") {
            return format!(
                "https://crates.io/search?q={}",
                urlencoding::encode(query.trim())
            );
        }
        if let Some(query) = trimmed.strip_prefix("gh:") {
            return format!(
                "https://github.com/search?q={}",
                urlencoding::encode(query.trim())
            );
        }
        if let Some(query) = trimmed.strip_prefix("cve:") {
            return format!(
                "https://nvd.nist.gov/vuln/search/results?form_type=Basic&results_type=overview&query={}",
                urlencoding::encode(query.trim())
            );
        }
        if let Some(query) = trimmed.strip_prefix("hn:") {
            return format!(
                "https://hn.algolia.com/?q={}",
                urlencoding::encode(query.trim())
            );
        }
        if let Some(query) = trimmed.strip_prefix("gitlab:") {
            return format!(
                "https://gitlab.com/explore/projects?name={}",
                urlencoding::encode(query.trim())
            );
        }

        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            return trimmed.to_string();
        }

        if trimmed.contains('.') && !trimmed.contains(' ') {
            return format!("https://{}", trimmed);
        }

        // Default to search engine
        format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(trimmed)
        )
    }

    /// Resolves relative URLs against a base URL.
    pub fn resolve_relative(base: &str, relative: &str) -> String {
        let rel = relative.trim();
        if rel.is_empty() {
            return base.to_string();
        }
        if rel.starts_with("http://") || rel.starts_with("https://") {
            return rel.to_string();
        }
        if rel.starts_with("//") {
            return format!("https:{}", rel);
        }

        let base_trimmed = base.trim();
        let (scheme_host, path_base) = if let Some(idx) = base_trimmed.find("://") {
            let rest = &base_trimmed[idx + 3..];
            if let Some(slash_idx) = rest.find('/') {
                let full_host = &base_trimmed[..idx + 3 + slash_idx];
                let full_path = &rest[slash_idx..];
                (full_host, full_path)
            } else {
                (base_trimmed, "/")
            }
        } else {
            ("https://localhost", "/")
        };

        if rel.starts_with('/') {
            format!("{}{}", scheme_host, rel)
        } else if rel.starts_with('?') {
            let base_without_query = base_trimmed.split('?').next().unwrap_or(base_trimmed);
            format!("{}{}", base_without_query, rel)
        } else if rel.starts_with('#') {
            let base_without_frag = base_trimmed.split('#').next().unwrap_or(base_trimmed);
            format!("{}{}", base_without_frag, rel)
        } else {
            let dir = if let Some(last_slash) = path_base.rfind('/') {
                &path_base[..last_slash + 1]
            } else {
                "/"
            };
            format!("{}{}{}", scheme_host, dir, rel)
        }
    }
}

// Minimal urlencoding utility to avoid extra external crate dependencies if needed
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut encoded = String::with_capacity(s.len() * 3);
        for b in s.bytes() {
            match b {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(b as char);
                }
                b' ' => encoded.push('+'),
                _ => {
                    encoded.push_str(&format!("%{:02X}", b));
                }
            }
        }
        encoded
    }
}

/// HTML cleaner and Reader Mode document transformer.
pub struct HtmlReaderEngine {
    pub wrap_width: usize,
}

impl Default for HtmlReaderEngine {
    fn default() -> Self {
        Self { wrap_width: 90 }
    }
}

impl HtmlReaderEngine {
    pub fn new(wrap_width: usize) -> Self {
        Self {
            wrap_width: wrap_width.max(30),
        }
    }

    /// Unescapes common and numeric HTML entities.
    pub fn unescape_entities(html: &str) -> String {
        let mut out = String::with_capacity(html.len());
        let mut chars = html.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '&' {
                let mut entity = String::new();
                let mut closed = false;
                while let Some(&next_c) = chars.peek() {
                    if next_c == ';' {
                        chars.next();
                        closed = true;
                        break;
                    } else if next_c.is_alphanumeric() || next_c == '#' {
                        entity.push(chars.next().unwrap());
                        if entity.len() > 10 {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                if closed {
                    match entity.as_str() {
                        "quot" => out.push('"'),
                        "amp" => out.push('&'),
                        "apos" => out.push('\''),
                        "lt" => out.push('<'),
                        "gt" => out.push('>'),
                        "nbsp" | "ensp" | "emsp" | "thinsp" => out.push(' '),
                        "copy" => out.push('©'),
                        "reg" => out.push('®'),
                        "trade" => out.push('™'),
                        "euro" => out.push('€'),
                        "pound" => out.push('£'),
                        "yen" => out.push('¥'),
                        "sect" => out.push('§'),
                        "deg" => out.push('°'),
                        "plusmn" => out.push('±'),
                        "laquo" => out.push('«'),
                        "raquo" => out.push('»'),
                        "ndash" => out.push('–'),
                        "mdash" => out.push('—'),
                        "lsquo" => out.push('‘'),
                        "rsquo" => out.push('’'),
                        "ldquo" => out.push('“'),
                        "rdquo" => out.push('”'),
                        "bull" => out.push('•'),
                        "hellip" => out.push('…'),
                        s if s.starts_with("#x") || s.starts_with("#X") => {
                            if let Ok(val) = u32::from_str_radix(&s[2..], 16) {
                                if let Some(ch) = char::from_u32(val) {
                                    out.push(ch);
                                } else {
                                    out.push('&');
                                    out.push_str(&entity);
                                    out.push(';');
                                }
                            } else {
                                out.push('&');
                                out.push_str(&entity);
                                out.push(';');
                            }
                        }
                        s if s.starts_with('#') => {
                            if let Ok(val) = s[1..].parse::<u32>() {
                                if let Some(ch) = char::from_u32(val) {
                                    out.push(ch);
                                } else {
                                    out.push('&');
                                    out.push_str(&entity);
                                    out.push(';');
                                }
                            } else {
                                out.push('&');
                                out.push_str(&entity);
                                out.push(';');
                            }
                        }
                        _ => {
                            out.push('&');
                            out.push_str(&entity);
                            out.push(';');
                        }
                    }
                } else {
                    out.push('&');
                    out.push_str(&entity);
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    /// Strips HTML tags and normalizes whitespace in text snippets.
    pub fn clean_text(input: &str) -> String {
        let unescaped = Self::unescape_entities(input);
        let mut clean = String::with_capacity(unescaped.len());
        let mut inside_tag = false;
        for c in unescaped.chars() {
            if c == '<' {
                inside_tag = true;
            } else if c == '>' {
                inside_tag = false;
            } else if !inside_tag {
                clean.push(c);
            }
        }
        clean
    }

    /// Strips unwanted HTML elements (<script>, <style>, <noscript>, <svg>, comments).
    pub fn sanitize_html(html: &str) -> String {
        let mut result = String::with_capacity(html.len());
        let mut i = 0;
        let bytes = html.as_bytes();
        let len = bytes.len();

        while i < len {
            if bytes[i] == b'<' {
                // Check comment <!-- ... -->
                if i + 4 <= len && &html[i..i + 4] == "<!--" {
                    if let Some(end) = html[i + 4..].find("-->") {
                        i += 4 + end + 3;
                        continue;
                    } else {
                        break;
                    }
                }

                // Check tags to drop completely with inner content: <script, <style, <svg, <noscript, <iframe
                let drop_tags = ["script", "style", "svg", "noscript", "iframe"];
                let mut dropped = false;
                for tag in drop_tags {
                    let start_tag = format!("<{}", tag);
                    if i + start_tag.len() <= len
                        && html[i..i + start_tag.len()].eq_ignore_ascii_case(&start_tag)
                    {
                        let close_tag = format!("</{}>", tag);
                        let close_tag_upper = format!("</{}>", tag.to_uppercase());
                        if let Some(end) = html[i..].find(&close_tag) {
                            i += end + close_tag.len();
                            dropped = true;
                            break;
                        } else if let Some(end) = html[i..].find(&close_tag_upper) {
                            i += end + close_tag_upper.len();
                            dropped = true;
                            break;
                        }
                    }
                }

                if dropped {
                    continue;
                }
            }

            result.push(html[i..].chars().next().unwrap());
            i += html[i..].chars().next().unwrap().len_utf8();
        }

        result
    }

    /// Extracts `<title>` from HTML document.
    pub fn extract_title(html: &str) -> Option<String> {
        let lower = html.to_lowercase();
        if let Some(start) = lower.find("<title>") {
            let after = &html[start + 7..];
            if let Some(end) = after.to_lowercase().find("</title>") {
                let raw_title = &after[..end];
                let unescaped = Self::unescape_entities(raw_title);
                let cleaned = unescaped.split_whitespace().collect::<Vec<_>>().join(" ");
                if !cleaned.is_empty() {
                    return Some(cleaned);
                }
            }
        }
        None
    }

    /// Converts raw HTML string into a structured `WebPage`.
    pub fn parse_html(&self, base_url: &str, html: &str) -> WebPage {
        let title = Self::extract_title(html).unwrap_or_else(|| {
            if let Some(host) = base_url.split("://").nth(1) {
                host.split('/').next().unwrap_or(base_url).to_string()
            } else {
                base_url.to_string()
            }
        });
        let cleaned_html = Self::sanitize_html(html);

        let mut page = WebPage::new(base_url, title);
        page.raw_html = Some(html.to_string());

        let mut link_counter = 0;
        let mut links = Vec::new();
        let mut lines = Vec::new();

        // Parse token stream
        let mut cur_spans: Vec<WebLineSpan> = Vec::new();
        let mut inside_pre = false;
        let mut inside_tag = false;
        let mut current_tag = String::new();
        let mut current_text = String::new();
        let mut tag_stack: Vec<String> = Vec::new();
        let mut current_href: Option<String> = None;
        let mut current_link_text = String::new();

        let flush_text = |text: &mut String,
                          spans: &mut Vec<WebLineSpan>,
                          tag_stack: &[String],
                          current_href: &Option<String>,
                          link_counter: &mut usize,
                          links: &mut Vec<WebLink>,
                          current_line_idx: usize,
                          base_url: &str| {
            if text.is_empty() {
                return;
            }
            let raw = Self::unescape_entities(text);
            text.clear();
            if raw.is_empty() {
                return;
            }

            if let Some(href) = current_href {
                *link_counter += 1;
                let link_id = *link_counter;
                let resolved_url = UrlHelper::resolve_relative(base_url, href);
                let link_display = format!("{} [{}]", raw.trim(), link_id);

                links.push(WebLink {
                    id: link_id,
                    text: raw.trim().to_string(),
                    url: resolved_url.clone(),
                    line_idx: current_line_idx,
                });

                spans.push(WebLineSpan::link(link_display, link_id, resolved_url));
            } else {
                // Determine style from tag stack
                let is_bold = tag_stack
                    .iter()
                    .any(|t| t == "b" || t == "strong" || t == "th");
                let is_italic = tag_stack.iter().any(|t| t == "i" || t == "em");
                let is_code = tag_stack.iter().any(|t| t == "code" || t == "pre" || t == "kbd");
                let header_level = tag_stack.iter().find_map(|t| {
                    if t.starts_with('h') && t.len() == 2 {
                        t[1..2].parse::<u8>().ok()
                    } else {
                        None
                    }
                });

                let style = if let Some(lvl) = header_level {
                    SpanStyle::Header(lvl)
                } else if is_code {
                    SpanStyle::Code
                } else if is_bold {
                    SpanStyle::Bold
                } else if is_italic {
                    SpanStyle::Italic
                } else {
                    SpanStyle::Normal
                };

                spans.push(WebLineSpan::new(raw, style));
            }
        };

        let mut chars = cleaned_html.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '<' {
                // Flush accumulated text before tag
                let line_idx = lines.len();
                flush_text(
                    &mut current_text,
                    &mut cur_spans,
                    &tag_stack,
                    &current_href,
                    &mut link_counter,
                    &mut links,
                    line_idx,
                    base_url,
                );

                inside_tag = true;
                current_tag.clear();
            } else if c == '>' && inside_tag {
                inside_tag = false;
                let tag_str = current_tag.trim();

                if let Some(tag_name) = tag_str.strip_prefix('/') {
                    // Closing tag
                    let t = tag_name.trim().to_lowercase();
                    if t == "pre" {
                        inside_pre = false;
                    }
                    if t == "a" {
                        current_href = None;
                        current_link_text.clear();
                    }

                    if let Some(pos) = tag_stack.iter().rposition(|x| x == &t) {
                        tag_stack.remove(pos);
                    }

                    // Block closing tags cause line breaks
                    if matches!(
                        t.as_str(),
                        "p" | "h1"
                            | "h2"
                            | "h3"
                            | "h4"
                            | "h5"
                            | "h6"
                            | "div"
                            | "article"
                            | "section"
                            | "li"
                            | "blockquote"
                            | "pre"
                            | "tr"
                    ) {
                        let line_type = match t.as_str() {
                            "h1" => LineType::Header(1),
                            "h2" => LineType::Header(2),
                            "h3" => LineType::Header(3),
                            "h4" => LineType::Header(4),
                            "h5" => LineType::Header(5),
                            "h6" => LineType::Header(6),
                            "li" => LineType::ListItem,
                            "blockquote" => LineType::Blockquote,
                            "pre" => LineType::CodeBlock,
                            _ => LineType::Paragraph,
                        };

                        if !cur_spans.is_empty() {
                            lines.push(WebLine::new(line_type, std::mem::take(&mut cur_spans)));
                            if matches!(t.as_str(), "h1" | "h2" | "h3" | "p" | "blockquote") {
                                lines.push(WebLine::empty());
                            }
                        }
                    }
                } else {
                    // Opening tag
                    let parts: Vec<&str> = tag_str.split_whitespace().collect();
                    let t = parts.first().unwrap_or(&"").to_lowercase();

                    if t == "pre" {
                        inside_pre = true;
                    }
                    if t == "hr" {
                        if !cur_spans.is_empty() {
                            lines.push(WebLine::from_spans(std::mem::take(&mut cur_spans)));
                        }
                        lines.push(WebLine::new(
                            LineType::HorizontalRule,
                            vec![WebLineSpan::muted("─".repeat(self.wrap_width))],
                        ));
                        lines.push(WebLine::empty());
                    } else if t == "br" {
                        if !cur_spans.is_empty() {
                            lines.push(WebLine::from_spans(std::mem::take(&mut cur_spans)));
                        } else {
                            lines.push(WebLine::empty());
                        }
                    } else if t == "a" {
                        // Extract href attribute
                        let href = Self::extract_attribute_value(tag_str, "href");
                        current_href = href;
                        tag_stack.push("a".to_string());
                    } else if t == "li" {
                        if !cur_spans.is_empty() {
                            lines.push(WebLine::from_spans(std::mem::take(&mut cur_spans)));
                        }
                        cur_spans.push(WebLineSpan::accent(" • "));
                        tag_stack.push(t);
                    } else if t == "blockquote" {
                        if !cur_spans.is_empty() {
                            lines.push(WebLine::from_spans(std::mem::take(&mut cur_spans)));
                        }
                        cur_spans.push(WebLineSpan::muted(" │ "));
                        tag_stack.push(t);
                    } else if !t.is_empty() && !t.ends_with('/') {
                        tag_stack.push(t);
                    }
                }
            } else if inside_tag {
                current_tag.push(c);
            } else if inside_pre {
                if c == '\n' {
                    let line_idx = lines.len();
                    flush_text(
                        &mut current_text,
                        &mut cur_spans,
                        &tag_stack,
                        &current_href,
                        &mut link_counter,
                        &mut links,
                        line_idx,
                        base_url,
                    );
                    lines.push(WebLine::new(
                        LineType::CodeBlock,
                        std::mem::take(&mut cur_spans),
                    ));
                } else {
                    current_text.push(c);
                }
            } else {
                // Normal flow text
                if c.is_whitespace() {
                    if !current_text.ends_with(' ') && !current_text.is_empty() {
                        current_text.push(' ');
                    }
                } else {
                    current_text.push(c);
                }
            }
        }

        // Flush remainder
        let line_idx = lines.len();
        flush_text(
            &mut current_text,
            &mut cur_spans,
            &tag_stack,
            &current_href,
            &mut link_counter,
            &mut links,
            line_idx,
            base_url,
        );
        if !cur_spans.is_empty() {
            lines.push(WebLine::from_spans(cur_spans));
        }

        // Wrap lines according to terminal width
        let wrapped_lines = self.wrap_document(lines);

        page.lines = wrapped_lines;
        page.links = links;
        page
    }

    /// Extracts attribute value (e.g. href="..." or href='...') from HTML tag string.
    pub fn extract_attribute_value(tag: &str, attr: &str) -> Option<String> {
        let lower = tag.to_lowercase();
        let target = format!("{}=", attr);
        if let Some(idx) = lower.find(&target) {
            let rest = tag[idx + target.len()..].trim_start();
            if rest.starts_with('"') {
                if let Some(end) = rest[1..].find('"') {
                    return Some(rest[1..1 + end].to_string());
                }
            } else if rest.starts_with('\'') {
                if let Some(end) = rest[1..].find('\'') {
                    return Some(rest[1..1 + end].to_string());
                }
            } else {
                let token = rest.split_whitespace().next().unwrap_or("");
                return Some(token.trim_end_matches('>').to_string());
            }
        }
        None
    }

    /// Wraps document lines to fit target column width nicely without breaking words.
    pub fn wrap_document(&self, lines: Vec<WebLine>) -> Vec<WebLine> {
        let mut result = Vec::with_capacity(lines.len());

        for line in lines {
            if line.line_type == LineType::CodeBlock || line.line_type == LineType::HorizontalRule {
                result.push(line);
                continue;
            }

            let total_len: usize = line.spans.iter().map(|s| s.text.width()).sum();
            if total_len <= self.wrap_width {
                result.push(line);
                continue;
            }

            // Wrap line spans
            let mut current_wrapped_spans = Vec::new();
            let mut current_width = 0;

            for span in line.spans {
                let words: Vec<&str> = span.text.split(' ').collect();
                let mut current_span_word_buf = String::new();

                for (w_idx, word) in words.iter().enumerate() {
                    let word_width = word.width() + if w_idx > 0 { 1 } else { 0 };

                    if current_width + word_width > self.wrap_width && current_width > 0 {
                        if !current_span_word_buf.is_empty() {
                            current_wrapped_spans.push(WebLineSpan::new(
                                std::mem::take(&mut current_span_word_buf),
                                span.style.clone(),
                            ));
                        }
                        result.push(WebLine::new(
                            line.line_type,
                            std::mem::take(&mut current_wrapped_spans),
                        ));
                        current_width = 0;
                    }

                    if w_idx > 0 && !current_span_word_buf.is_empty() {
                        current_span_word_buf.push(' ');
                        current_width += 1;
                    }
                    current_span_word_buf.push_str(word);
                    current_width += word.width();
                }

                if !current_span_word_buf.is_empty() {
                    current_wrapped_spans.push(WebLineSpan::new(
                        current_span_word_buf,
                        span.style.clone(),
                    ));
                }
            }

            if !current_wrapped_spans.is_empty() {
                result.push(WebLine::new(line.line_type, current_wrapped_spans));
            }
        }

        result
    }
}

/// HTTP Client wrapper for web fetching.
#[derive(Debug, Clone)]
pub struct WebFetcher {
    client: reqwest::blocking::Client,
}

impl Default for WebFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl WebFetcher {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:120.0) Gecko/20100101 Firefox/120.0 QwxWebReader/1.0")
            .timeout(std::time::Duration::from_secs(12))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        Self { client }
    }

    /// Fetches a web page by URL, parses it, and records latency.
    pub fn fetch(&self, url: &str, wrap_width: usize) -> Result<WebPage, String> {
        let target_url = UrlHelper::resolve(url);
        let start_time = Instant::now();

        let response = self
            .client
            .get(&target_url)
            .header("Accept", "text/html,application/xhtml+xml,text/plain;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "fr,fr-FR;q=0.8,en-US;q=0.5,en;q=0.3")
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("text/html")
            .to_string();

        let final_url = response.url().to_string();
        let body = response
            .text()
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        let duration = start_time.elapsed().as_millis();

        let engine = HtmlReaderEngine::new(wrap_width);
        let mut page = engine.parse_html(&final_url, &body);
        page.status_code = status;
        page.content_type = content_type;
        page.fetch_duration_ms = duration;

        Ok(page)
    }
}

/// Main Terminal Web Browser / Reader controller.
#[derive(Debug, Clone)]
pub struct WebBrowser {
    pub current_page: Option<WebPage>,
    pub history: WebHistory,
    pub bookmarks: Vec<WebBookmark>,
    pub scroll_offset: usize,
    pub selected_link_idx: Option<usize>,
    pub view_mode: WebReaderViewMode,
    pub is_loading: bool,
    pub status_message: Option<String>,
    pub url_input: String,
    pub url_prompt_active: bool,
    pub link_input: String,
    pub link_prompt_active: bool,
    pub search_query: String,
    pub search_mode: bool,
    pub search_matches: Vec<usize>,
    pub active_search_match: usize,
    pub fetcher: WebFetcher,
}

impl Default for WebBrowser {
    fn default() -> Self {
        Self::new()
    }
}

impl WebBrowser {
    pub fn new() -> Self {
        let mut bookmarks = Vec::new();
        bookmarks.push(WebBookmark {
            title: "Rust Standard Library".to_string(),
            url: "https://doc.rust-lang.org/std/".to_string(),
            tags: vec!["rust".to_string(), "docs".to_string()],
        });
        bookmarks.push(WebBookmark {
            title: "Crates.io".to_string(),
            url: "https://crates.io".to_string(),
            tags: vec!["rust".to_string(), "packages".to_string()],
        });
        bookmarks.push(WebBookmark {
            title: "DuckDuckGo".to_string(),
            url: "https://html.duckduckgo.com/html/".to_string(),
            tags: vec!["search".to_string()],
        });
        bookmarks.push(WebBookmark {
            title: "Hacker News".to_string(),
            url: "https://news.ycombinator.com".to_string(),
            tags: vec!["news".to_string(), "tech".to_string()],
        });

        Self {
            current_page: None,
            history: WebHistory::new(),
            bookmarks,
            scroll_offset: 0,
            selected_link_idx: None,
            view_mode: WebReaderViewMode::Reader,
            is_loading: false,
            status_message: Some("Ready. Press 'o' to open URL, 'b' back, 'f' forward, 'l' links.".to_string()),
            url_input: String::new(),
            url_prompt_active: false,
            link_input: String::new(),
            link_prompt_active: false,
            search_query: String::new(),
            search_mode: false,
            search_matches: Vec::new(),
            active_search_match: 0,
            fetcher: WebFetcher::new(),
        }
    }

    /// Opens and loads a URL.
    pub fn open_url(&mut self, url: &str, terminal_width: u16) {
        self.is_loading = true;
        self.status_message = Some(format!("Fetching {}...", url));

        let width = if terminal_width > 10 {
            (terminal_width - 6) as usize
        } else {
            80
        };

        match self.fetcher.fetch(url, width) {
            Ok(page) => {
                let final_url = page.url.clone();
                self.history.push(final_url.clone());
                self.status_message = Some(format!(
                    "Loaded '{}' in {}ms ({} links)",
                    page.title,
                    page.fetch_duration_ms,
                    page.links.len()
                ));
                self.current_page = Some(page);
                self.scroll_offset = 0;
                self.selected_link_idx = None;
                self.search_matches.clear();
            }
            Err(e) => {
                self.status_message = Some(format!("Error loading URL: {}", e));
            }
        }
        self.is_loading = false;
    }

    /// Reloads the current web page.
    pub fn reload(&mut self, terminal_width: u16) {
        if let Some(current_url) = self.history.current().cloned() {
            self.open_url(&current_url, terminal_width);
        }
    }

    /// Navigates back in history.
    pub fn go_back(&mut self, terminal_width: u16) {
        if self.history.can_go_back() {
            if let Some(url) = self.history.back().cloned() {
                self.open_url(&url, terminal_width);
            }
        } else {
            self.status_message = Some("No previous page in history.".to_string());
        }
    }

    /// Navigates forward in history.
    pub fn go_forward(&mut self, terminal_width: u16) {
        if self.history.can_go_forward() {
            if let Some(url) = self.history.forward().cloned() {
                self.open_url(&url, terminal_width);
            }
        } else {
            self.status_message = Some("No forward page in history.".to_string());
        }
    }

    /// Scrolls down by a given amount of lines.
    pub fn scroll_down(&mut self, lines: usize) {
        if let Some(ref page) = self.current_page {
            let total = page.total_lines();
            if self.scroll_offset + lines < total {
                self.scroll_offset += lines;
            } else if total > 0 {
                self.scroll_offset = total.saturating_sub(1);
            }
        }
    }

    /// Scrolls up by a given amount of lines.
    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    /// Cycles to the next hyperlink on the page.
    pub fn next_link(&mut self) {
        if let Some(ref page) = self.current_page {
            if page.links.is_empty() {
                return;
            }
            let next_idx = match self.selected_link_idx {
                Some(current) => (current + 1) % page.links.len(),
                None => 0,
            };
            self.selected_link_idx = Some(next_idx);

            // Auto-scroll to selected link if it is offscreen
            if let Some(link) = page.links.get(next_idx) {
                if link.line_idx < self.scroll_offset {
                    self.scroll_offset = link.line_idx;
                } else if link.line_idx > self.scroll_offset + 20 {
                    self.scroll_offset = link.line_idx.saturating_sub(5);
                }
                self.status_message = Some(format!("Link [{}]: {} -> {}", link.id, link.text, link.url));
            }
        }
    }

    /// Cycles to the previous hyperlink on the page.
    pub fn prev_link(&mut self) {
        if let Some(ref page) = self.current_page {
            if page.links.is_empty() {
                return;
            }
            let prev_idx = match self.selected_link_idx {
                Some(0) | None => page.links.len().saturating_sub(1),
                Some(current) => current - 1,
            };
            self.selected_link_idx = Some(prev_idx);

            if let Some(link) = page.links.get(prev_idx) {
                if link.line_idx < self.scroll_offset {
                    self.scroll_offset = link.line_idx;
                } else if link.line_idx > self.scroll_offset + 20 {
                    self.scroll_offset = link.line_idx.saturating_sub(5);
                }
                self.status_message = Some(format!("Link [{}]: {} -> {}", link.id, link.text, link.url));
            }
        }
    }

    /// Follows the currently selected hyperlink.
    pub fn follow_selected_link(&mut self, terminal_width: u16) {
        if let Some(ref page) = self.current_page {
            if let Some(idx) = self.selected_link_idx {
                if let Some(link) = page.links.get(idx) {
                    let url = link.url.clone();
                    self.open_url(&url, terminal_width);
                }
            }
        }
    }

    /// Follows a hyperlink by its numeric identifier `[1]`, `[2]`, etc.
    pub fn follow_link_by_id(&mut self, id: usize, terminal_width: u16) {
        if let Some(ref page) = self.current_page {
            if let Some(link) = page.find_link_by_id(id) {
                let url = link.url.clone();
                self.open_url(&url, terminal_width);
            } else {
                self.status_message = Some(format!("Link [{}] not found on this page.", id));
            }
        }
    }

    /// Opens a `SearchResultItem` from the `search` module in the web reader.
    pub fn open_search_result(&mut self, item: &crate::search::SearchResultItem, terminal_width: u16) {
        if !item.url.is_empty() {
            self.open_url(&item.url, terminal_width);
        } else if let Some(ref content) = item.raw_content {
            let width = if terminal_width > 10 {
                (terminal_width - 6) as usize
            } else {
                80
            };
            let engine = HtmlReaderEngine::new(width);
            let mut page = engine.parse_html("local://search-item", content);
            page.title = item.title.clone();
            self.current_page = Some(page);
            self.scroll_offset = 0;
            self.selected_link_idx = None;
            self.status_message = Some(format!("Viewing '{}'", item.title));
        }
    }

    /// Loads search results into the web reader as a formatted, interactive WebPage.
    pub fn load_search_results(
        &mut self,
        query: &str,
        provider_name: &str,
        results: &[crate::search::SearchResultItem],
        terminal_width: u16,
    ) {
        let width = if terminal_width > 10 {
            (terminal_width - 6) as usize
        } else {
            80
        };
        let page = WebPage::from_search_results(query, provider_name, results, width);
        let url = page.url.clone();
        self.history.push(url);
        self.current_page = Some(page);
        self.scroll_offset = 0;
        self.selected_link_idx = None;
        self.search_matches.clear();
        self.status_message = Some(format!(
            "Loaded {} search result(s) for '{}' [{}]",
            results.len(),
            query,
            provider_name
        ));
    }

    /// Performs search with a given `SearchProvider` and displays results as a WebPage.
    pub fn search_with_provider(
        &mut self,
        provider: crate::search::SearchProvider,
        query: &str,
        terminal_width: u16,
        current_dir: &std::path::Path,
    ) {
        let mut hub = crate::search::SearchHub::new();
        hub.active_provider = provider;
        hub.query = query.to_string();
        hub.perform_search(current_dir);
        self.load_search_results(query, provider.name(), &hub.results, terminal_width);
    }

    /// Performs text search within the loaded document.
    pub fn execute_search(&mut self, query: &str) {
        self.search_query = query.to_string();
        if let Some(ref page) = self.current_page {
            self.search_matches = page.search_text(query);
            self.active_search_match = 0;
            if let Some(&first_line) = self.search_matches.first() {
                self.scroll_offset = first_line.saturating_sub(2);
                self.status_message = Some(format!(
                    "Found {} matches for '{}'. Match 1/{}",
                    self.search_matches.len(),
                    query,
                    self.search_matches.len()
                ));
            } else {
                self.status_message = Some(format!("No matches found for '{}'.", query));
            }
        }
    }

    /// Alias for execute_search
    pub fn search_page(&mut self, query: &str) {
        self.execute_search(query);
    }

    /// Jumps to the next search match.
    pub fn next_search_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.active_search_match = (self.active_search_match + 1) % self.search_matches.len();
        let target_line = self.search_matches[self.active_search_match];
        self.scroll_offset = target_line.saturating_sub(2);
        self.status_message = Some(format!(
            "Match {}/{} on line {}",
            self.active_search_match + 1,
            self.search_matches.len(),
            target_line + 1
        ));
    }

    /// Jumps to the previous search match.
    pub fn prev_search_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        if self.active_search_match == 0 {
            self.active_search_match = self.search_matches.len() - 1;
        } else {
            self.active_search_match -= 1;
        }
        let target_line = self.search_matches[self.active_search_match];
        self.scroll_offset = target_line.saturating_sub(2);
        self.status_message = Some(format!(
            "Match {}/{} on line {}",
            self.active_search_match + 1,
            self.search_matches.len(),
            target_line + 1
        ));
    }

    /// Adds current page to bookmarks.
    pub fn bookmark_current_page(&mut self) {
        if let Some(ref page) = self.current_page {
            let url = page.url.clone();
            if !self.bookmarks.iter().any(|b| b.url == url) {
                self.bookmarks.push(WebBookmark {
                    title: page.title.clone(),
                    url: url.clone(),
                    tags: Vec::new(),
                });
                self.status_message = Some(format!("Bookmarked '{}'.", page.title));
            } else {
                self.status_message = Some("Page is already bookmarked.".to_string());
            }
        }
    }

    /// Toggles view mode between Reader, Links Index, and Raw Source.
    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            WebReaderViewMode::Reader => WebReaderViewMode::LinksList,
            WebReaderViewMode::LinksList => WebReaderViewMode::RawSource,
            WebReaderViewMode::RawSource => WebReaderViewMode::Reader,
        };
        self.scroll_offset = 0;
        self.status_message = Some(format!("Switched to {}", self.view_mode.name()));
    }

    /// Draws the complete terminal web interface using crossterm queue.
    pub fn draw<W: Write>(&self, writer: &mut W, width: u16, height: u16) -> io::Result<()> {
        let w = width as usize;
        let h = height as usize;

        if w < 20 || h < 6 {
            return Ok(());
        }

        let bg_main = Color::Rgb { r: 18, g: 20, b: 28 };
        let fg_normal = Color::Rgb { r: 210, g: 220, b: 235 };
        let fg_muted = Color::Rgb { r: 110, g: 120, b: 145 };
        let bg_header = Color::Rgb { r: 28, g: 32, b: 45 };
        let fg_accent = Color::Rgb { r: 130, g: 180, b: 250 };
        let fg_link = Color::Rgb { r: 100, g: 210, b: 240 };
        let bg_link_sel = Color::Rgb { r: 70, g: 60, b: 120 };
        let fg_code = Color::Rgb { r: 140, g: 220, b: 180 };
        let bg_code = Color::Rgb { r: 25, g: 30, b: 40 };

        // 1. Draw Header / URL Bar
        queue!(writer, MoveTo(0, 0), SetBackgroundColor(bg_header), SetForegroundColor(fg_accent))?;
        let title_str = self
            .current_page
            .as_ref()
            .map(|p| p.title.as_str())
            .unwrap_or("QWX Web Reader");
        let truncated_title = if title_str.width() > w.saturating_sub(30) {
            format!("{}…", &title_str[..w.saturating_sub(32)])
        } else {
            title_str.to_string()
        };

        let mode_badge = format!(" [{}] ", self.view_mode.name());
        let header_left = format!(" 🌐 QWX Web | {} ", truncated_title);
        let header_space = w.saturating_sub(header_left.width() + mode_badge.width());
        queue!(
            writer,
            Print(header_left),
            SetForegroundColor(fg_muted),
            Print(" ".repeat(header_space)),
            SetForegroundColor(Color::Rgb { r: 240, g: 190, b: 110 }),
            Print(mode_badge),
            ResetColor
        )?;

        // 2. Draw URL bar / Navigation info
        queue!(writer, MoveTo(0, 1), SetBackgroundColor(bg_header), SetForegroundColor(fg_normal))?;
        let current_url = self
            .current_page
            .as_ref()
            .map(|p| p.url.as_str())
            .unwrap_or("about:blank");
        let back_ind = if self.history.can_go_back() { "◄" } else { "◁" };
        let fwd_ind = if self.history.can_go_forward() { "►" } else { "▷" };

        let url_bar = format!(" {} {} URL: {} ", back_ind, fwd_ind, current_url);
        let url_bar_padded = if url_bar.width() < w {
            format!("{}{}", url_bar, " ".repeat(w - url_bar.width()))
        } else {
            format!("{}… ", &url_bar[..w.saturating_sub(2)])
        };
        queue!(writer, Print(url_bar_padded), ResetColor)?;

        // 3. Separator
        queue!(writer, MoveTo(0, 2), SetForegroundColor(Color::Rgb { r: 45, g: 52, b: 72 }))?;
        queue!(writer, Print("─".repeat(w)), ResetColor)?;

        // 4. Main Body Content Rendering
        let content_start_y = 3;
        let content_height = h.saturating_sub(5); // leave 2 lines for status & prompts

        match self.view_mode {
            WebReaderViewMode::Reader => {
                if let Some(ref page) = self.current_page {
                    let total_lines = page.lines.len();
                    for y in 0..content_height {
                        let line_idx = self.scroll_offset + y;
                        queue!(writer, MoveTo(0, (content_start_y + y) as u16), SetBackgroundColor(bg_main))?;

                        if line_idx < total_lines {
                            let line = &page.lines[line_idx];
                            let mut current_col = 0;

                            // Left padding
                            queue!(writer, SetForegroundColor(fg_muted), Print("  "))?;
                            current_col += 2;

                            for span in &line.spans {
                                if current_col >= w.saturating_sub(1) {
                                    break;
                                }

                                match &span.style {
                                    SpanStyle::Header(level) => {
                                        let h_color = match level {
                                            1 => Color::Rgb { r: 255, g: 215, b: 120 },
                                            2 => Color::Rgb { r: 160, g: 200, b: 255 },
                                            3 => Color::Rgb { r: 180, g: 160, b: 240 },
                                            _ => Color::Rgb { r: 200, g: 200, b: 200 },
                                        };
                                        queue!(writer, SetForegroundColor(h_color), Print(&span.text))?;
                                    }
                                    SpanStyle::Code => {
                                        queue!(
                                            writer,
                                            SetBackgroundColor(bg_code),
                                            SetForegroundColor(fg_code),
                                            Print(&span.text),
                                            SetBackgroundColor(bg_main)
                                        )?;
                                    }
                                    SpanStyle::Bold => {
                                        queue!(
                                            writer,
                                            SetForegroundColor(Color::Rgb { r: 250, g: 250, b: 255 }),
                                            Print(&span.text)
                                        )?;
                                    }
                                    SpanStyle::Italic => {
                                        queue!(
                                            writer,
                                            SetForegroundColor(Color::Rgb { r: 190, g: 205, b: 220 }),
                                            Print(&span.text)
                                        )?;
                                    }
                                    SpanStyle::Link { id, .. } => {
                                        let is_sel = self.selected_link_idx.and_then(|idx| page.links.get(idx)).map(|l| l.id) == Some(*id);
                                        if is_sel {
                                            queue!(
                                                writer,
                                                SetBackgroundColor(bg_link_sel),
                                                SetForegroundColor(Color::Rgb { r: 255, g: 255, b: 255 }),
                                                Print(&span.text),
                                                SetBackgroundColor(bg_main)
                                            )?;
                                        } else {
                                            queue!(writer, SetForegroundColor(fg_link), Print(&span.text))?;
                                        }
                                    }
                                    SpanStyle::Accent => {
                                        queue!(writer, SetForegroundColor(fg_accent), Print(&span.text))?;
                                    }
                                    SpanStyle::Muted => {
                                        queue!(writer, SetForegroundColor(fg_muted), Print(&span.text))?;
                                    }
                                    _ => {
                                        queue!(writer, SetForegroundColor(fg_normal), Print(&span.text))?;
                                    }
                                }
                                current_col += span.text.width();
                            }

                            // Fill rest of line with spaces
                            if current_col < w {
                                queue!(writer, Print(" ".repeat(w - current_col)))?;
                            }
                        } else {
                            // Blank line
                            queue!(writer, Print(" ".repeat(w)))?;
                        }
                    }
                } else {
                    // Empty / Welcome state
                    for y in 0..content_height {
                        queue!(writer, MoveTo(0, (content_start_y + y) as u16), SetBackgroundColor(bg_main))?;
                        if y == 2 {
                            let msg = "  🌐 Bienvenue dans le mode Web Reader de QWX";
                            queue!(writer, SetForegroundColor(fg_accent), Print(msg), Print(" ".repeat(w.saturating_sub(msg.width()))))?;
                        } else if y == 4 {
                            let msg = "  Raccourcis clavier :";
                            queue!(writer, SetForegroundColor(Color::Rgb { r: 240, g: 200, b: 120 }), Print(msg), Print(" ".repeat(w.saturating_sub(msg.width()))))?;
                        } else if y == 5 {
                            let msg = "    • 'o' ou 'g' : Ouvrir une URL ou chercher (ex: ddg:rust, crates:tokio, https://...)";
                            queue!(writer, SetForegroundColor(fg_normal), Print(msg), Print(" ".repeat(w.saturating_sub(msg.width()))))?;
                        } else if y == 6 {
                            let msg = "    • 'Tab' / 'Shift-Tab' : Naviguer de lien en lien";
                            queue!(writer, SetForegroundColor(fg_normal), Print(msg), Print(" ".repeat(w.saturating_sub(msg.width()))))?;
                        } else if y == 7 {
                            let msg = "    • 'Enter' : Suivre le lien sélectionné | 'f' : Taper le numéro du lien [N]";
                            queue!(writer, SetForegroundColor(fg_normal), Print(msg), Print(" ".repeat(w.saturating_sub(msg.width()))))?;
                        } else if y == 8 {
                            let msg = "    • 'b' : Page précédente (Back) | 'Shift-f' : Page suivante (Forward) | 'r' : Recharger";
                            queue!(writer, SetForegroundColor(fg_normal), Print(msg), Print(" ".repeat(w.saturating_sub(msg.width()))))?;
                        } else if y == 9 {
                            let msg = "    • '/' : Rechercher dans la page | 'n' / 'N' : Occurrence suivante/précédente";
                            queue!(writer, SetForegroundColor(fg_normal), Print(msg), Print(" ".repeat(w.saturating_sub(msg.width()))))?;
                        } else if y == 10 {
                            let msg = "    • 'm' : Basculer le mode d'affichage (Reader / Index des Liens / Code Source)";
                            queue!(writer, SetForegroundColor(fg_normal), Print(msg), Print(" ".repeat(w.saturating_sub(msg.width()))))?;
                        } else if y == 11 {
                            let msg = "    • 'B' : Ajouter la page aux favoris (Bookmarks)";
                            queue!(writer, SetForegroundColor(fg_normal), Print(msg), Print(" ".repeat(w.saturating_sub(msg.width()))))?;
                        } else if y == 13 {
                            let msg = "  Signets prédéfinis :";
                            queue!(writer, SetForegroundColor(Color::Rgb { r: 240, g: 200, b: 120 }), Print(msg), Print(" ".repeat(w.saturating_sub(msg.width()))))?;
                        } else if y >= 14 && y < 14 + self.bookmarks.len() {
                            let b_idx = y - 14;
                            let bm = &self.bookmarks[b_idx];
                            let msg = format!("    [{}] {} ({})", b_idx + 1, bm.title, bm.url);
                            queue!(writer, SetForegroundColor(fg_link), Print(&msg), Print(" ".repeat(w.saturating_sub(msg.width()))))?;
                        } else {
                            queue!(writer, Print(" ".repeat(w)))?;
                        }
                    }
                }
            }
            WebReaderViewMode::LinksList => {
                let empty_vec = Vec::new();
                let links = self.current_page.as_ref().map(|p| &p.links).unwrap_or(&empty_vec);
                for y in 0..content_height {
                    let idx = self.scroll_offset + y;
                    queue!(writer, MoveTo(0, (content_start_y + y) as u16), SetBackgroundColor(bg_main))?;
                    if idx < links.len() {
                        let link = &links[idx];
                        let is_sel = self.selected_link_idx == Some(idx);
                        let prefix = if is_sel { " ▶ " } else { "   " };
                        let line_str = format!("{}[{:>3}] {:<40} -> {}", prefix, link.id, link.text, link.url);
                        let color = if is_sel { Color::Rgb { r: 255, g: 255, b: 255 } } else { fg_link };
                        let bg = if is_sel { bg_link_sel } else { bg_main };
                        queue!(
                            writer,
                            SetBackgroundColor(bg),
                            SetForegroundColor(color),
                            Print(&line_str),
                            Print(" ".repeat(w.saturating_sub(line_str.width()))),
                            SetBackgroundColor(bg_main)
                        )?;
                    } else {
                        queue!(writer, Print(" ".repeat(w)))?;
                    }
                }
            }
            WebReaderViewMode::RawSource => {
                let raw_lines: Vec<&str> = self
                    .current_page
                    .as_ref()
                    .and_then(|p| p.raw_html.as_deref())
                    .map(|h| h.lines().collect())
                    .unwrap_or_default();

                for y in 0..content_height {
                    let idx = self.scroll_offset + y;
                    queue!(writer, MoveTo(0, (content_start_y + y) as u16), SetBackgroundColor(bg_main))?;
                    if idx < raw_lines.len() {
                        let line_str = format!(" {:>5} │ {}", idx + 1, raw_lines[idx]);
                        let trun = if line_str.width() > w { &line_str[..w] } else { &line_str };
                        queue!(
                            writer,
                            SetForegroundColor(fg_muted),
                            Print(trun),
                            Print(" ".repeat(w.saturating_sub(trun.width())))
                        )?;
                    } else {
                        queue!(writer, Print(" ".repeat(w)))?;
                    }
                }
            }
        }

        // 5. Scrollbar / Progress indicator
        if let Some(ref page) = self.current_page {
            let total = match self.view_mode {
                WebReaderViewMode::Reader => page.total_lines(),
                WebReaderViewMode::LinksList => page.links.len(),
                WebReaderViewMode::RawSource => page.raw_html.as_ref().map(|h| h.lines().count()).unwrap_or(0),
            };
            if total > content_height {
                let pct = ((self.scroll_offset + content_height).min(total) * 100) / total;
                let pct_badge = format!(" {}% [{}/{}] ", pct, self.scroll_offset + 1, total);
                let badge_x = w.saturating_sub(pct_badge.width() + 2);
                queue!(
                    writer,
                    MoveTo(badge_x as u16, (content_start_y + content_height - 1) as u16),
                    SetBackgroundColor(bg_header),
                    SetForegroundColor(fg_muted),
                    Print(pct_badge),
                    ResetColor
                )?;
            }
        }

        // 6. Interactive Prompts (URL bar, Link jump, Search)
        let prompt_y = (h.saturating_sub(2)) as u16;
        if self.url_prompt_active {
            queue!(
                writer,
                MoveTo(0, prompt_y),
                SetBackgroundColor(Color::Rgb { r: 35, g: 45, b: 65 }),
                SetForegroundColor(Color::Rgb { r: 255, g: 215, b: 120 }),
                Print(" 🌐 Enter URL / Search: "),
                SetForegroundColor(Color::Rgb { r: 255, g: 255, b: 255 }),
                Print(&self.url_input),
                Print("█"),
                Print(" ".repeat(w.saturating_sub(25 + self.url_input.width()))),
                ResetColor
            )?;
        } else if self.link_prompt_active {
            queue!(
                writer,
                MoveTo(0, prompt_y),
                SetBackgroundColor(Color::Rgb { r: 35, g: 45, b: 65 }),
                SetForegroundColor(fg_link),
                Print(" 🔗 Jump to Link ID [#]: "),
                SetForegroundColor(Color::Rgb { r: 255, g: 255, b: 255 }),
                Print(&self.link_input),
                Print("█"),
                Print(" ".repeat(w.saturating_sub(27 + self.link_input.width()))),
                ResetColor
            )?;
        } else if self.search_mode {
            queue!(
                writer,
                MoveTo(0, prompt_y),
                SetBackgroundColor(Color::Rgb { r: 35, g: 45, b: 65 }),
                SetForegroundColor(Color::Rgb { r: 160, g: 240, b: 180 }),
                Print(" 🔍 Search: "),
                SetForegroundColor(Color::Rgb { r: 255, g: 255, b: 255 }),
                Print(&self.search_query),
                Print("█"),
                Print(" ".repeat(w.saturating_sub(14 + self.search_query.width()))),
                ResetColor
            )?;
        } else {
            // Status bar
            queue!(
                writer,
                MoveTo(0, prompt_y),
                SetBackgroundColor(bg_header),
                SetForegroundColor(fg_muted)
            )?;
            let msg = self.status_message.as_deref().unwrap_or("Ready.");
            let status_line = format!(" ℹ {}", msg);
            let padded = if status_line.width() < w {
                format!("{}{}", status_line, " ".repeat(w - status_line.width()))
            } else {
                format!("{}… ", &status_line[..w.saturating_sub(2)])
            };
            queue!(writer, Print(padded), ResetColor)?;
        }

        // 7. Footer / Keybinds quick guide
        let footer_y = (h.saturating_sub(1)) as u16;
        queue!(
            writer,
            MoveTo(0, footer_y),
            SetBackgroundColor(Color::Rgb { r: 15, g: 18, b: 25 }),
            SetForegroundColor(fg_muted)
        )?;
        let keybinds_text = " [o] Open | [b] Back | [f] Jump Link | [Tab] Next Link | [/] Search | [m] View Mode | [r] Reload | [q] Quit";
        let padded_footer = if keybinds_text.width() < w {
            format!("{}{}", keybinds_text, " ".repeat(w - keybinds_text.width()))
        } else {
            format!("{}…", &keybinds_text[..w.saturating_sub(1)])
        };
        queue!(writer, Print(padded_footer), ResetColor)?;

        writer.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unescape_entities() {
        let input = "Hello &amp; welcome to &quot;Rust&quot; &lt;2026&gt; &copy; &euro;";
        let output = HtmlReaderEngine::unescape_entities(input);
        assert_eq!(output, "Hello & welcome to \"Rust\" <2026> © €");
    }

    #[test]
    fn test_numeric_entities() {
        let input = "&#65;&#66;&#67; &#x44;&#x45;&#x46;";
        let output = HtmlReaderEngine::unescape_entities(input);
        assert_eq!(output, "ABC DEF");
    }

    #[test]
    fn test_sanitize_html_scripts_and_styles() {
        let html = "<div><script>alert('hack');</script><p>Clean Text</p><style>body{color:red;}</style></div>";
        let sanitized = HtmlReaderEngine::sanitize_html(html);
        assert!(!sanitized.contains("alert"));
        assert!(!sanitized.contains("color:red"));
        assert!(sanitized.contains("Clean Text"));
    }

    #[test]
    fn test_extract_title() {
        let html = "<html><head><title>My Awesome Terminal Page</title></head><body><h1>Hi</h1></body></html>";
        let title = HtmlReaderEngine::extract_title(html);
        assert_eq!(title, Some("My Awesome Terminal Page".to_string()));
    }

    #[test]
    fn test_url_helper_shortcuts() {
        assert_eq!(
            UrlHelper::resolve("ddg:rust lang"),
            "https://html.duckduckgo.com/html/?q=rust+lang"
        );
        assert_eq!(
            UrlHelper::resolve("crates:tokio"),
            "https://crates.io/search?q=tokio"
        );
        assert_eq!(
            UrlHelper::resolve("wiki:Linux"),
            "https://en.wikipedia.org/wiki/Special:Search?search=Linux"
        );
        assert_eq!(
            UrlHelper::resolve("gh:qwx"),
            "https://github.com/search?q=qwx"
        );
        assert_eq!(
            UrlHelper::resolve("example.com"),
            "https://example.com"
        );
    }

    #[test]
    fn test_url_relative_resolution() {
        assert_eq!(
            UrlHelper::resolve_relative("https://example.com/docs/intro", "/api/item"),
            "https://example.com/api/item"
        );
        assert_eq!(
            UrlHelper::resolve_relative("https://example.com/docs/intro", "section.html"),
            "https://example.com/docs/section.html"
        );
    }

    #[test]
    fn test_html_parsing_and_link_extraction() {
        let html = r#"
        <html>
            <head><title>Test Article</title></head>
            <body>
                <h1>Main Heading</h1>
                <p>This is a paragraph with a <a href="https://example.com/learn">Link to Learn</a> and <b>bold text</b>.</p>
                <ul>
                    <li>Item 1</li>
                    <li>Item 2</li>
                </ul>
                <pre><code>let x = 42;</code></pre>
            </body>
        </html>
        "#;

        let engine = HtmlReaderEngine::new(80);
        let page = engine.parse_html("https://example.com", html);

        assert_eq!(page.title, "Test Article");
        assert_eq!(page.links.len(), 1);
        assert_eq!(page.links[0].id, 1);
        assert_eq!(page.links[0].text, "Link to Learn");
        assert_eq!(page.links[0].url, "https://example.com/learn");

        let search_res = page.search_text("Heading");
        assert!(!search_res.is_empty());
    }

    #[test]
    fn test_web_history_navigation() {
        let mut history = WebHistory::new();
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());

        history.push("https://page1.com");
        history.push("https://page2.com");
        history.push("https://page3.com");

        assert_eq!(history.current(), Some(&"https://page3.com".to_string()));
        assert!(history.can_go_back());

        assert_eq!(history.back(), Some(&"https://page2.com".to_string()));
        assert_eq!(history.back(), Some(&"https://page1.com".to_string()));
        assert!(!history.can_go_back());
        assert!(history.can_go_forward());

        assert_eq!(history.forward(), Some(&"https://page2.com".to_string()));
    }

    #[test]
    fn test_browser_bookmarks() {
        let mut browser = WebBrowser::new();
        let initial_len = browser.bookmarks.len();
        browser.current_page = Some(WebPage::new("https://doc.rust-lang.org/book/", "The Rust Book"));
        browser.bookmark_current_page();

        assert_eq!(browser.bookmarks.len(), initial_len + 1);
        assert_eq!(browser.bookmarks.last().unwrap().title, "The Rust Book");
    }

    #[test]
    fn test_web_search_integration() {
        let results = vec![
            crate::search::SearchResultItem {
                provider: crate::search::SearchProvider::GitHub,
                title: "qwx-editor".to_string(),
                description: "Rust 2x2 modal text editor".to_string(),
                url: "https://github.com/saigo/qwx".to_string(),
                extra_info: "★ 100".to_string(),
                clone_url: Some("https://github.com/saigo/qwx.git".to_string()),
                raw_content: None,
            },
        ];

        let page = WebPage::from_search_results("qwx", "GitHub", &results, 80);
        assert_eq!(page.title, "Search: qwx (GitHub)");
        assert_eq!(page.links.len(), 1);
        assert_eq!(page.links[0].text, "qwx-editor");
        assert_eq!(page.links[0].url, "https://github.com/saigo/qwx");

        let mut browser = WebBrowser::new();
        browser.load_search_results("qwx", "GitHub", &results, 80);
        assert!(browser.current_page.is_some());
        assert_eq!(browser.current_page.as_ref().unwrap().links.len(), 1);

        assert_eq!(
            UrlHelper::search_provider_for_query("gh:qwx"),
            Some((crate::search::SearchProvider::GitHub, "qwx"))
        );
        assert_eq!(
            UrlHelper::search_provider_for_query("cve:openssl"),
            Some((crate::search::SearchProvider::Cve, "openssl"))
        );
    }
}
