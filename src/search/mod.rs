use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Filter providers for the search engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchProvider {
    All,
    GitHub,
    GitLab,
    Wikipedia,
    Cve,
    HackerNews,
    LocalAudit,
    Web,
}

impl SearchProvider {
    pub fn all_variants() -> &'static [SearchProvider] {
        &[
            SearchProvider::All,
            SearchProvider::GitHub,
            SearchProvider::GitLab,
            SearchProvider::Wikipedia,
            SearchProvider::Cve,
            SearchProvider::HackerNews,
            SearchProvider::LocalAudit,
            SearchProvider::Web,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            SearchProvider::All => "All",
            SearchProvider::GitHub => "GitHub",
            SearchProvider::GitLab => "GitLab",
            SearchProvider::Wikipedia => "Wikipedia",
            SearchProvider::Cve => "CVE / Security",
            SearchProvider::HackerNews => "Hacker News",
            SearchProvider::LocalAudit => "Local Audit",
            SearchProvider::Web => "Web / DuckDuckGo",
        }
    }

    pub fn shortcut_key(&self) -> char {
        match self {
            SearchProvider::All => '1',
            SearchProvider::GitHub => '2',
            SearchProvider::GitLab => '3',
            SearchProvider::Wikipedia => '4',
            SearchProvider::Cve => '5',
            SearchProvider::HackerNews => '6',
            SearchProvider::LocalAudit => '7',
            SearchProvider::Web => '8',
        }
    }
}

/// Clone progress information for live UI feedback
#[derive(Debug, Clone, Default)]
pub struct CloneProgress {
    pub percentage: u8,
    pub indexed_objects: usize,
    pub total_objects: usize,
    pub received_bytes: usize,
    pub current_step: String,
}

impl CloneProgress {
    pub fn new(step: impl Into<String>) -> Self {
        Self {
            percentage: 0,
            indexed_objects: 0,
            total_objects: 0,
            received_bytes: 0,
            current_step: step.into(),
        }
    }

    pub fn format_progress_bar(&self, width: usize) -> String {
        let bar_width = width.saturating_sub(18).max(10);
        let filled_width = (bar_width * self.percentage as usize) / 100;
        let empty_width = bar_width.saturating_sub(filled_width);

        let filled = "█".repeat(filled_width);
        let empty = "░".repeat(empty_width);

        format!("[{}{}] {:>3}%", filled, empty, self.percentage)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub provider: SearchProvider,
    pub title: String,
    pub description: String,
    pub url: String,
    pub extra_info: String,
    pub clone_url: Option<String>,
    pub raw_content: Option<String>,
}

/// Active modal or prompt for interactive Git / PR actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionPrompt {
    CloneRepo {
        repo_url: String,
        dest_input: String,
    },
    CloneInProgress {
        repo_url: String,
        dest_path: String,
        progress_pct: u8,
        status_text: String,
    },
    CreateBranch {
        branch_input: String,
    },
    CheckoutBranch {
        branch_input: String,
    },
    ExportReport {
        path_input: String,
    },
    CreatePullRequest {
        repo_input: String,
        title_input: String,
        body_input: String,
        head_input: String,
        base_input: String,
        token_input: String,
        step: usize, // 0: repo, 1: title, 2: body, 3: head, 4: base, 5: token
    },
}

/// Search hub state and engine
#[derive(Debug, Clone)]
pub struct SearchHub {
    pub query: String,
    pub active_provider: SearchProvider,
    pub results: Vec<SearchResultItem>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub preview_scroll: usize,
    pub status_message: Option<String>,
    pub is_loading: bool,
    pub prompt: Option<ActionPrompt>,
    pub web_browser: crate::web::WebBrowser,
    pub show_web_reader: bool,
}

impl Default for SearchHub {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchHub {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            active_provider: SearchProvider::All,
            results: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            preview_scroll: 0,
            status_message: Some("Press Enter to search, Tab to filter, 'w' for Web Reader, 'c' to clone, 'b' to create branch, 'p' for PR, 'a' to audit local CVEs.".to_string()),
            is_loading: false,
            prompt: None,
            web_browser: crate::web::WebBrowser::new(),
            show_web_reader: false,
        }
    }

    /// Open selected result item in the embedded Web Reader
    pub fn open_selected_in_web_reader(&mut self, terminal_width: u16) {
        if let Some(item) = self.selected_item().cloned() {
            self.web_browser.open_search_result(&item, terminal_width);
            self.show_web_reader = true;
            self.status_message = Some(format!(
                "Reading '{}' in Web Reader. Press [Esc] or [q] to return.",
                item.title
            ));
        } else {
            self.status_message = Some("No item selected to open in Web Reader.".to_string());
        }
    }

    /// View all current search results as an interactive Web Page inside the Web Reader
    pub fn view_results_as_web_page(&mut self, terminal_width: u16) {
        if !self.results.is_empty() {
            let provider_name = self.active_provider.name();
            let query = if self.query.trim().is_empty() {
                "all"
            } else {
                self.query.trim()
            };
            self.web_browser.load_search_results(
                query,
                provider_name,
                &self.results,
                terminal_width,
            );
            self.show_web_reader = true;
            self.status_message = Some(
                "Viewing search results in Web Reader. Press [Esc] or [q] to return.".to_string(),
            );
        } else {
            self.status_message = Some("No search results to view as Web Page.".to_string());
        }
    }

    /// Close the embedded Web Reader and return to SearchHub results grid
    pub fn close_web_reader(&mut self) {
        self.show_web_reader = false;
        self.status_message = Some("Returned to Search Hub.".to_string());
    }

    /// Check if currently viewing the web reader
    pub fn is_viewing_web(&self) -> bool {
        self.show_web_reader
    }

    pub fn set_provider(&mut self, provider: SearchProvider) {
        self.active_provider = provider;
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.preview_scroll = 0;
    }

    pub fn next_provider(&mut self) {
        let variants = SearchProvider::all_variants();
        let current_pos = variants
            .iter()
            .position(|&p| p == self.active_provider)
            .unwrap_or(0);
        let next_pos = (current_pos + 1) % variants.len();
        self.set_provider(variants[next_pos]);
    }

    pub fn prev_provider(&mut self) {
        let variants = SearchProvider::all_variants();
        let current_pos = variants
            .iter()
            .position(|&p| p == self.active_provider)
            .unwrap_or(0);
        let prev_pos = if current_pos == 0 {
            variants.len() - 1
        } else {
            current_pos - 1
        };
        self.set_provider(variants[prev_pos]);
    }

    pub fn next_result(&mut self) {
        if !self.results.is_empty() && self.selected_index + 1 < self.results.len() {
            self.selected_index += 1;
            self.preview_scroll = 0;
        }
    }

    pub fn prev_result(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.preview_scroll = 0;
        }
    }

    pub fn scroll_preview_down(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_add(3);
    }

    pub fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(3);
    }

    pub fn selected_item(&self) -> Option<&SearchResultItem> {
        self.results.get(self.selected_index)
    }

    /// Perform online search based on active provider and query string.
    pub fn perform_search(&mut self, current_dir: &Path) {
        let query_clean = self.query.trim();

        // Check if query starts with provider shortcut prefix like "gh:", "gitlab:", "wiki:", "cve:", "hn:", "audit:", "web:", "ddg:"
        let (provider, effective_query) = if let Some(rest) = query_clean.strip_prefix("gh:") {
            (SearchProvider::GitHub, rest.trim())
        } else if let Some(rest) = query_clean.strip_prefix("gitlab:") {
            (SearchProvider::GitLab, rest.trim())
        } else if let Some(rest) = query_clean.strip_prefix("wiki:") {
            (SearchProvider::Wikipedia, rest.trim())
        } else if let Some(rest) = query_clean.strip_prefix("cve:") {
            (SearchProvider::Cve, rest.trim())
        } else if let Some(rest) = query_clean.strip_prefix("hn:") {
            (SearchProvider::HackerNews, rest.trim())
        } else if let Some(rest) = query_clean.strip_prefix("audit:") {
            (SearchProvider::LocalAudit, rest.trim())
        } else if let Some(rest) = query_clean.strip_prefix("web:") {
            (SearchProvider::Web, rest.trim())
        } else if let Some(rest) = query_clean.strip_prefix("ddg:") {
            (SearchProvider::Web, rest.trim())
        } else {
            (self.active_provider, query_clean)
        };

        self.is_loading = true;
        self.results.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.preview_scroll = 0;

        match provider {
            SearchProvider::All => {
                if !effective_query.is_empty() {
                    let mut all_res = Vec::new();
                    all_res.extend(search_github(effective_query));
                    all_res.extend(search_gitlab(effective_query));
                    all_res.extend(search_cve(effective_query));
                    all_res.extend(search_hacker_news(effective_query));
                    all_res.extend(search_wikipedia(effective_query));
                    all_res.extend(search_duckduckgo(effective_query));
                    self.results = all_res;
                } else {
                    self.results = audit_local_workspace(current_dir);
                }
            }
            SearchProvider::GitHub => {
                self.results = search_github(effective_query);
            }
            SearchProvider::GitLab => {
                self.results = search_gitlab(effective_query);
            }
            SearchProvider::Wikipedia => {
                self.results = search_wikipedia(effective_query);
            }
            SearchProvider::Cve => {
                if effective_query.is_empty() {
                    self.results = audit_local_workspace(current_dir);
                } else {
                    self.results = search_cve(effective_query);
                }
            }
            SearchProvider::HackerNews => {
                self.results = search_hacker_news(effective_query);
            }
            SearchProvider::LocalAudit => {
                self.results = audit_local_workspace(current_dir);
            }
            SearchProvider::Web => {
                self.results = search_duckduckgo(effective_query);
            }
        }

        self.is_loading = false;
        let count = self.results.len();
        if count == 0 {
            self.status_message = Some(format!(
                "No results found for '{}' [{}]",
                effective_query,
                provider.name()
            ));
        } else {
            self.status_message = Some(format!(
                "{} result(s) found for '{}' [{}]",
                count,
                effective_query,
                provider.name()
            ));
        }
    }

    /// Clone the selected repository or an arbitrary clone URL
    pub fn start_clone_selected(&mut self) {
        if let Some(item) = self.selected_item() {
            let clone_url = item.clone_url.clone().unwrap_or_else(|| {
                if item.url.contains("github.com") || item.url.contains("gitlab.com") {
                    item.url.clone()
                } else {
                    String::new()
                }
            });

            if !clone_url.is_empty() {
                // Suggest destination directory name based on repo
                let repo_name = clone_url
                    .trim_end_matches('/')
                    .trim_end_matches(".git")
                    .rsplit('/')
                    .next()
                    .unwrap_or("repo")
                    .to_string();
                self.prompt = Some(ActionPrompt::CloneRepo {
                    repo_url: clone_url,
                    dest_input: repo_name,
                });
                self.status_message = Some("Enter target directory for cloning and press Enter (or Esc to cancel):".to_string());
            } else {
                self.status_message = Some("Selected item does not have a valid git repository URL to clone.".to_string());
            }
        } else {
            self.status_message = Some("Please select a repository to clone first.".to_string());
        }
    }

    /// Prompt to create a new branch in the current workspace or selected repo
    pub fn start_create_branch(&mut self) {
        self.prompt = Some(ActionPrompt::CreateBranch {
            branch_input: String::new(),
        });
        self.status_message = Some("Enter new git branch name (or Esc to cancel):".to_string());
    }

    /// Prompt to create a Pull Request on GitHub
    pub fn start_create_pull_request(&mut self) {
        let repo_guess = if let Some(item) = self.selected_item() {
            if item.provider == SearchProvider::GitHub {
                item.title.clone()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        self.prompt = Some(ActionPrompt::CreatePullRequest {
            repo_input: repo_guess,
            title_input: String::new(),
            body_input: String::new(),
            head_input: String::new(),
            base_input: "main".to_string(),
            token_input: String::new(),
            step: 0,
        });
        self.status_message = Some("Create Pull Request - Step 1/6 : Repository (owner/repo):".to_string());
    }

    /// Prompt to checkout / switch a git branch
    pub fn start_checkout_branch(&mut self) {
        self.prompt = Some(ActionPrompt::CheckoutBranch {
            branch_input: String::new(),
        });
        self.status_message = Some("Enter branch name to checkout / switch (or Esc to cancel):".to_string());
    }

    /// Prompt to export results or audit to a Markdown file
    pub fn start_export_report(&mut self) {
        let default_name = match self.active_provider {
            SearchProvider::LocalAudit | SearchProvider::Cve => "cve-security-report.md",
            _ => "search-report.md",
        };
        self.prompt = Some(ActionPrompt::ExportReport {
            path_input: default_name.to_string(),
        });
        self.status_message = Some("Enter filename for Markdown export (or Esc to cancel):".to_string());
    }

    /// Open the selected item URL in the default system web browser
    pub fn open_selected_in_browser(&mut self) {
        if let Some(item) = self.selected_item() {
            let res = open_url_in_browser(&item.url);
            match res {
                Ok(msg) => self.status_message = Some(msg),
                Err(err) => self.status_message = Some(format!("Error: {}", err)),
            }
        } else {
            self.status_message = Some("No item selected to open.".to_string());
        }
    }

    /// Draw the complete search hub interface in a refined 2x2 grid structure
    pub fn draw<W: Write>(
        &self,
        w: &mut W,
        start_x: u16,
        start_y: u16,
        width: u16,
        height: u16,
    ) -> io::Result<()> {
        if self.show_web_reader {
            return self.web_browser.draw(w, width, height);
        }

        if width < 30 || height < 10 {
            return Ok(());
        }

        // Refined Dark Theme Palette
        let bg_color = Color::Black;
        let header_bg = Color::Rgb {
            r: 16,
            g: 20,
            b: 28,
        };
        let tab_active_bg = Color::Rgb {
            r: 130,
            g: 110,
            b: 190, // Cosmic violet from theme
        };
        let tab_inactive_bg = Color::Rgb {
            r: 24,
            g: 28,
            b: 38,
        };
        let text_color = Color::Rgb {
            r: 220,
            g: 225,
            b: 240,
        };
        let text_dim = Color::Rgb {
            r: 110,
            g: 120,
            b: 140,
        };
        let highlight_bg = Color::Rgb {
            r: 35,
            g: 48,
            b: 75,
        };
        let border_color = Color::Rgb {
            r: 45,
            g: 52,
            b: 70, // FINDER_BORDER theme color
        };
        let border_focus = Color::Rgb {
            r: 100,
            g: 170,
            b: 255,
        };
        let title_color = Color::Rgb {
            r: 80,
            g: 200,
            b: 240,
        };
        let accent_gold = Color::Rgb {
            r: 255,
            g: 215,
            b: 90,
        };
        let accent_green = Color::Rgb {
            r: 100,
            g: 220,
            b: 150,
        };

        // Fill background with deep black
        let empty_line = " ".repeat(width as usize);
        for row in 0..height {
            queue!(
                w,
                MoveTo(start_x, start_y + row),
                SetBackgroundColor(bg_color),
                SetForegroundColor(text_color),
                Print(&empty_line)
            )?;
        }

        // 1. Header & Title Bar
        queue!(
            w,
            MoveTo(start_x, start_y),
            SetBackgroundColor(header_bg),
            SetForegroundColor(title_color),
            Print(" ◈ QWX SEARCH ENGINE & DEV HUB ◈ ")
        )?;
        let title_len = " ◈ QWX SEARCH ENGINE & DEV HUB ◈ ".width() as u16;
        let tag_text = " [2x2 Grid Hub] ";
        let tag_len = tag_text.width() as u16;
        if width > title_len + tag_len {
            let padding_len = width - title_len - tag_len;
            let padding = " ".repeat(padding_len as usize);
            queue!(
                w,
                Print(padding),
                SetForegroundColor(text_dim),
                Print(tag_text)
            )?;
        } else if width > title_len {
            let padding = " ".repeat((width - title_len) as usize);
            queue!(w, Print(padding))?;
        }

        // 2. Filter Tabs
        let mut tab_x = start_x + 1;
        let tab_y = start_y + 1;
        queue!(w, MoveTo(tab_x, tab_y), SetBackgroundColor(bg_color))?;

        for provider in SearchProvider::all_variants() {
            let is_active = *provider == self.active_provider;
            let tab_text = format!(" {}:{} ", provider.shortcut_key(), provider.name());
            let tab_width = tab_text.width() as u16;

            if tab_x + tab_width > start_x + width - 1 {
                break;
            }

            if is_active {
                queue!(
                    w,
                    MoveTo(tab_x, tab_y),
                    SetBackgroundColor(tab_active_bg),
                    SetForegroundColor(Color::White),
                    Print(&tab_text)
                )?;
            } else {
                queue!(
                    w,
                    MoveTo(tab_x, tab_y),
                    SetBackgroundColor(tab_inactive_bg),
                    SetForegroundColor(text_dim),
                    Print(&tab_text)
                )?;
            }
            tab_x += tab_width + 1;
        }

        // 3. Search Input Bar
        let search_y = start_y + 2;
        let search_prompt = format!(" 🔍 [{}] Search: {} ", self.active_provider.name(), self.query);
        let cursor_char = if self.prompt.is_none() { "█" } else { " " };
        let full_search = format!("{}{}", search_prompt, cursor_char);

        queue!(
            w,
            MoveTo(start_x + 1, search_y),
            SetBackgroundColor(Color::Rgb {
                r: 18,
                g: 22,
                b: 32
            }),
            SetForegroundColor(accent_gold),
            Print(&full_search)
        )?;
        let search_rendered_width = full_search.width() as u16;
        if width.saturating_sub(2) > search_rendered_width {
            let pad = " ".repeat((width.saturating_sub(2) - search_rendered_width) as usize);
            queue!(w, Print(pad))?;
        }

        // 4. 2x2 Grid Layout Calculation
        let grid_top_y = start_y + 3;
        let grid_height = height.saturating_sub(6).max(6);
        let half_width = (width / 2).max(20).min(width.saturating_sub(15));
        let left_inner_w = (half_width.saturating_sub(1)) as usize;
        let right_inner_w = (width.saturating_sub(half_width).saturating_sub(2)) as usize;

        let row_top_h = (grid_height / 2).max(3);
        let row_bot_h = grid_height.saturating_sub(row_top_h).max(3);
        let middle_y = grid_top_y + row_top_h;
        let bottom_y = grid_top_y + grid_height;

        let inner_top_h = (row_top_h.saturating_sub(1)) as usize;
        let inner_bot_h = (row_bot_h.saturating_sub(1)) as usize;

        // --- DRAW 2x2 BORDERS ---
        // 4.1 Top border
        let left_border_line = "─".repeat(left_inner_w);
        let right_border_line = "─".repeat(right_inner_w);
        queue!(
            w,
            MoveTo(start_x, grid_top_y),
            SetBackgroundColor(bg_color),
            SetForegroundColor(border_color),
            Print("┌"),
            Print(&left_border_line),
            Print("┬"),
            Print(&right_border_line),
            Print("┐")
        )?;

        // Top labels in border
        let count_str = if self.results.is_empty() {
            String::new()
        } else {
            format!(" ({}/{})", self.selected_index + 1, self.results.len())
        };
        let top_left_title = format!(" ◈ 1. SEARCH RESULTS{} ", count_str);
        let top_right_title = " ◈ 2. RESOURCE OVERVIEW ";

        queue!(
            w,
            MoveTo(start_x + 2, grid_top_y),
            SetForegroundColor(title_color),
            Print(&top_left_title),
            MoveTo(start_x + half_width + 2, grid_top_y),
            SetForegroundColor(accent_gold),
            Print(top_right_title)
        )?;

        // 4.2 Top half vertical dividers
        for r in 1..row_top_h {
            let y = grid_top_y + r;
            queue!(
                w,
                MoveTo(start_x, y),
                SetForegroundColor(border_color),
                Print("│"),
                MoveTo(start_x + half_width, y),
                Print("│"),
                MoveTo(start_x + width - 1, y),
                Print("│")
            )?;
        }

        // 4.3 Middle horizontal divider
        queue!(
            w,
            MoveTo(start_x, middle_y),
            SetForegroundColor(border_color),
            Print("├"),
            Print(&left_border_line),
            Print("┼"),
            Print(&right_border_line),
            Print("┤")
        )?;

        // Middle labels in divider
        let bot_left_title = " ◈ 3. DEV TOOLS & PROVIDERS ";
        let scroll_info = if self.preview_scroll > 0 {
            format!(" [▲ Line {}] ", self.preview_scroll + 1)
        } else {
            " ".to_string()
        };
        let bot_right_title = format!(" ◈ 4. CONTENT & CODE PREVIEW{} ", scroll_info);

        queue!(
            w,
            MoveTo(start_x + 2, middle_y),
            SetForegroundColor(Color::Rgb {
                r: 160,
                g: 140,
                b: 240,
            }),
            Print(bot_left_title),
            MoveTo(start_x + half_width + 2, middle_y),
            SetForegroundColor(accent_green),
            Print(&bot_right_title)
        )?;

        // 4.4 Bottom half vertical dividers
        for r in 1..row_bot_h {
            let y = middle_y + r;
            queue!(
                w,
                MoveTo(start_x, y),
                SetForegroundColor(border_color),
                Print("│"),
                MoveTo(start_x + half_width, y),
                Print("│"),
                MoveTo(start_x + width - 1, y),
                Print("│")
            )?;
        }

        // 4.5 Bottom border
        queue!(
            w,
            MoveTo(start_x, bottom_y),
            SetForegroundColor(border_color),
            Print("└"),
            Print(&left_border_line),
            Print("┴"),
            Print(&right_border_line),
            Print("┘")
        )?;

        // --- RENDER QUADRANT 1 (Top-Left): SEARCH RESULTS ---
        let q1_x = start_x + 1;
        let q1_y = grid_top_y + 1;
        let q1_w = left_inner_w.saturating_sub(1);
        let q1_h = inner_top_h;

        if self.results.is_empty() {
            let empty_msg = if self.is_loading {
                match self.active_provider {
                    SearchProvider::GitHub => "⚡ Searching GitHub repositories & issues...",
                    SearchProvider::GitLab => "⚡ Searching GitLab projects & repositories...",
                    SearchProvider::Wikipedia => "🌐 Fetching Wikipedia articles & docs...",
                    SearchProvider::Cve => "🛡 Scanning & analyzing CVE vulnerabilities...",
                    SearchProvider::HackerNews => "📡 Fetching Hacker News discussions...",
                    SearchProvider::LocalAudit => "📦 Auditing dependencies & scanning CVEs...",
                    SearchProvider::Web => "🌐 Searching web via DuckDuckGo...",
                    SearchProvider::All => "⚙ Multi-source live search in progress...",
                }
            } else {
                "No results. Type a query and press [Enter]."
            };

            queue!(
                w,
                MoveTo(q1_x + 1, q1_y + 1),
                SetBackgroundColor(bg_color),
                SetForegroundColor(if self.is_loading {
                    accent_gold
                } else {
                    text_dim
                }),
                Print(empty_msg)
            )?;
        } else {
            let visible_count = q1_h;
            let mut scroll_offset = self.scroll_offset;
            if self.selected_index < scroll_offset {
                scroll_offset = self.selected_index;
            } else if self.selected_index >= scroll_offset + visible_count {
                scroll_offset = self.selected_index + 1 - visible_count;
            }

            for (idx, item) in self
                .results
                .iter()
                .skip(scroll_offset)
                .take(visible_count)
                .enumerate()
            {
                let absolute_idx = scroll_offset + idx;
                let is_selected = absolute_idx == self.selected_index;
                let line_y = q1_y + idx as u16;

                let icon = match item.provider {
                    SearchProvider::GitHub => "",
                    SearchProvider::GitLab => "",
                    SearchProvider::Wikipedia => "󰖬",
                    SearchProvider::Cve => "🛡",
                    SearchProvider::HackerNews => "",
                    SearchProvider::LocalAudit => "🔍",
                    SearchProvider::Web => "🌐",
                    SearchProvider::All => "◆",
                };

                let pointer = if is_selected { "▶" } else { " " };
                let item_line = format!("{} {} {}", pointer, icon, item.title);
                let truncated_title = truncate_to_width(&item_line, q1_w);

                if is_selected {
                    queue!(
                        w,
                        MoveTo(q1_x, line_y),
                        SetBackgroundColor(highlight_bg),
                        SetForegroundColor(Color::White),
                        Print(&truncated_title)
                    )?;
                    let tw = truncated_title.width() as u16;
                    if (q1_w as u16) > tw {
                        let pad = " ".repeat((q1_w as u16 - tw) as usize);
                        queue!(w, Print(pad))?;
                    }
                } else {
                    queue!(
                        w,
                        MoveTo(q1_x, line_y),
                        SetBackgroundColor(bg_color),
                        SetForegroundColor(text_color),
                        Print(&truncated_title)
                    )?;
                }
            }
        }

        // --- RENDER QUADRANT 2 (Top-Right): RESOURCE OVERVIEW ---
        let q2_x = start_x + half_width + 1;
        let q2_y = grid_top_y + 1;
        let q2_w = right_inner_w.saturating_sub(1);
        let q2_h = inner_top_h;

        if let Some(selected) = self.selected_item() {
            let mut overview_lines = Vec::new();
            overview_lines.push((format!("• Title   : {}", selected.title), Color::White));
            overview_lines.push((format!("• Source  : {}", selected.provider.name()), title_color));
            if !selected.extra_info.is_empty() {
                overview_lines.push((format!("• Metrics : {}", selected.extra_info), accent_gold));
            }
            if !selected.url.is_empty() {
                overview_lines.push((format!("• URL     : {}", selected.url), Color::Rgb { r: 100, g: 190, b: 255 }));
            }
            if let Some(ref clone_url) = selected.clone_url {
                overview_lines.push((format!("• GitClone: {}", clone_url), accent_green));
            }
            overview_lines.push(("• Actions : [w] WebReader  [o] ExtBrowser  [c] Clone  [p] PR".to_string(), text_dim));

            for (idx, (line, color)) in overview_lines.iter().take(q2_h).enumerate() {
                let line_y = q2_y + idx as u16;
                let truncated = truncate_to_width(line, q2_w);
                queue!(
                    w,
                    MoveTo(q2_x, line_y),
                    SetBackgroundColor(bg_color),
                    SetForegroundColor(*color),
                    Print(&truncated)
                )?;
            }
        } else {
            queue!(
                w,
                MoveTo(q2_x + 1, q2_y + 1),
                SetBackgroundColor(bg_color),
                SetForegroundColor(text_dim),
                Print("Select an item to view detailed resource metadata.")
            )?;
        }

        // --- RENDER QUADRANT 3 (Bottom-Left): DEV TOOLS & PROVIDERS ---
        let q3_x = start_x + 1;
        let q3_y = middle_y + 1;
        let q3_w = left_inner_w.saturating_sub(1);
        let q3_h = inner_bot_h;

        let mut provider_lines: Vec<(String, Color)> = Vec::new();
        provider_lines.push(("Providers & Shortcuts:".to_string(), title_color));
        provider_lines.push((
            format!(
                "  {} [1] All Sources    {} [2] GitHub",
                if self.active_provider == SearchProvider::All { "●" } else { "○" },
                if self.active_provider == SearchProvider::GitHub { "●" } else { "○" }
            ),
            if self.active_provider == SearchProvider::All || self.active_provider == SearchProvider::GitHub {
                Color::White
            } else {
                text_dim
            },
        ));
        provider_lines.push((
            format!(
                "  {} [3] GitLab         {} [4] Wikipedia",
                if self.active_provider == SearchProvider::GitLab { "●" } else { "○" },
                if self.active_provider == SearchProvider::Wikipedia { "●" } else { "○" }
            ),
            if self.active_provider == SearchProvider::GitLab || self.active_provider == SearchProvider::Wikipedia {
                Color::White
            } else {
                text_dim
            },
        ));
        provider_lines.push((
            format!(
                "  {} [5] CVE Security   {} [6] Hacker News",
                if self.active_provider == SearchProvider::Cve { "●" } else { "○" },
                if self.active_provider == SearchProvider::HackerNews { "●" } else { "○" }
            ),
            if self.active_provider == SearchProvider::Cve || self.active_provider == SearchProvider::HackerNews {
                Color::White
            } else {
                text_dim
            },
        ));
        provider_lines.push((
            format!(
                "  {} [7] Local Audit    {} [8] Web (DuckDuckGo)",
                if self.active_provider == SearchProvider::LocalAudit { "●" } else { "○" },
                if self.active_provider == SearchProvider::Web { "●" } else { "○" }
            ),
            if self.active_provider == SearchProvider::LocalAudit || self.active_provider == SearchProvider::Web {
                accent_gold
            } else {
                text_dim
            },
        ));
        provider_lines.push(("Dev Actions & Web:".to_string(), accent_gold));
        provider_lines.push(("  [w] Open in Web Reader   [v] View as Web Page".to_string(), text_color));
        provider_lines.push(("  [c] Clone Repo           [b] Create Branch".to_string(), text_color));
        provider_lines.push(("  [s] Switch Branch        [p] Create Pull Request".to_string(), text_color));
        provider_lines.push(("  [e] Export Report        [a] Audit Local CVEs".to_string(), text_color));

        for (idx, (line, color)) in provider_lines.iter().take(q3_h).enumerate() {
            let line_y = q3_y + idx as u16;
            let truncated = truncate_to_width(line, q3_w);
            queue!(
                w,
                MoveTo(q3_x, line_y),
                SetBackgroundColor(bg_color),
                SetForegroundColor(*color),
                Print(&truncated)
            )?;
        }

        // --- RENDER QUADRANT 4 (Bottom-Right): CONTENT & CODE PREVIEW ---
        let q4_x = start_x + half_width + 1;
        let q4_y = middle_y + 1;
        let q4_w = right_inner_w.saturating_sub(1);
        let q4_h = inner_bot_h;

        if let Some(selected) = self.selected_item() {
            let mut preview_lines = Vec::new();
            if !selected.description.is_empty() {
                preview_lines.push("Description:".to_string());
                for line in selected.description.lines() {
                    preview_lines.push(format!("  {}", line));
                }
            }

            if let Some(ref raw) = selected.raw_content {
                preview_lines.push("─".repeat(q4_w.min(40)));
                preview_lines.push("Details / Advisory / Body:".to_string());
                for line in raw.lines() {
                    preview_lines.push(format!("  {}", line));
                }
            }

            if preview_lines.is_empty() {
                preview_lines.push("No additional description or body available.".to_string());
            }

            for (idx, line) in preview_lines
                .iter()
                .skip(self.preview_scroll)
                .take(q4_h)
                .enumerate()
            {
                let line_y = q4_y + idx as u16;
                let truncated = truncate_to_width(line, q4_w);
                let is_header = line.starts_with("Description:") || line.starts_with("Details / Advisory");
                queue!(
                    w,
                    MoveTo(q4_x, line_y),
                    SetBackgroundColor(bg_color),
                    SetForegroundColor(if is_header {
                        accent_green
                    } else {
                        text_color
                    }),
                    Print(&truncated)
                )?;
            }
        } else {
            queue!(
                w,
                MoveTo(q4_x + 1, q4_y + 1),
                SetBackgroundColor(bg_color),
                SetForegroundColor(text_dim),
                Print("Preview documentation, CVE advisories, or repository summaries here.")
            )?;
        }

        // 5. Action Dialog / Modal Prompt (if active)
        if let Some(ref prompt) = self.prompt {
            let modal_w = width.min(74);
            let modal_h = 7;
            let modal_x = start_x + (width.saturating_sub(modal_w)) / 2;
            let modal_y = start_y + (height.saturating_sub(modal_h)) / 2;

            for r in 0..modal_h {
                queue!(
                    w,
                    MoveTo(modal_x, modal_y + r),
                    SetBackgroundColor(Color::Rgb {
                        r: 16,
                        g: 20,
                        b: 30
                    }),
                    Print(" ".repeat(modal_w as usize))
                )?;
            }

            // Draw modal box border
            queue!(
                w,
                MoveTo(modal_x, modal_y),
                SetForegroundColor(border_focus),
                Print(format!("┌{}┐", "─".repeat(modal_w as usize - 2)))
            )?;
            for r in 1..modal_h - 1 {
                queue!(
                    w,
                    MoveTo(modal_x, modal_y + r),
                    Print("│"),
                    MoveTo(modal_x + modal_w - 1, modal_y + r),
                    Print("│")
                )?;
            }
            queue!(
                w,
                MoveTo(modal_x, modal_y + modal_h - 1),
                Print(format!("└{}┘", "─".repeat(modal_w as usize - 2)))
            )?;

            match prompt {
                ActionPrompt::CloneRepo {
                    repo_url,
                    dest_input,
                } => {
                    queue!(
                        w,
                        MoveTo(modal_x + 2, modal_y + 1),
                        SetForegroundColor(accent_gold),
                        Print("📦 CLONE GIT REPOSITORY"),
                        MoveTo(modal_x + 2, modal_y + 2),
                        SetForegroundColor(Color::White),
                        Print(format!("URL   : {}", truncate_to_width(repo_url, modal_w as usize - 12))),
                        MoveTo(modal_x + 2, modal_y + 3),
                        SetForegroundColor(accent_green),
                        Print(format!("Target: {}█", dest_input)),
                        MoveTo(modal_x + 2, modal_y + 5),
                        SetForegroundColor(text_dim),
                        Print("[Enter] Start Clone | [Esc] Cancel")
                    )?;
                }
                ActionPrompt::CloneInProgress {
                    repo_url,
                    dest_path,
                    progress_pct,
                    status_text,
                } => {
                    let bar_w = (modal_w as usize).saturating_sub(14).max(10);
                    let filled = (bar_w * (*progress_pct as usize)) / 100;
                    let empty = bar_w.saturating_sub(filled);
                    let progress_bar = format!("[{}{}] {:>3}%", "█".repeat(filled), "░".repeat(empty), progress_pct);

                    queue!(
                        w,
                        MoveTo(modal_x + 2, modal_y + 1),
                        SetForegroundColor(title_color),
                        Print("🚀 CLONING IN PROGRESS..."),
                        MoveTo(modal_x + 2, modal_y + 2),
                        SetForegroundColor(Color::White),
                        Print(format!("Repository: {}", truncate_to_width(repo_url, modal_w as usize - 14))),
                        MoveTo(modal_x + 2, modal_y + 3),
                        SetForegroundColor(accent_gold),
                        Print(format!("Target    : {}", truncate_to_width(dest_path, modal_w as usize - 14))),
                        MoveTo(modal_x + 2, modal_y + 4),
                        SetForegroundColor(accent_green),
                        Print(&progress_bar),
                        MoveTo(modal_x + 2, modal_y + 5),
                        SetForegroundColor(text_dim),
                        Print(truncate_to_width(status_text, modal_w as usize - 6))
                    )?;
                }
                ActionPrompt::CreateBranch { branch_input } => {
                    queue!(
                        w,
                        MoveTo(modal_x + 2, modal_y + 1),
                        SetForegroundColor(accent_gold),
                        Print("🌿 CREATE GIT BRANCH"),
                        MoveTo(modal_x + 2, modal_y + 3),
                        SetForegroundColor(accent_green),
                        Print(format!("Branch name: {}█", branch_input)),
                        MoveTo(modal_x + 2, modal_y + 5),
                        SetForegroundColor(text_dim),
                        Print("[Enter] Create Branch | [Esc] Cancel")
                    )?;
                }
                ActionPrompt::CheckoutBranch { branch_input } => {
                    queue!(
                        w,
                        MoveTo(modal_x + 2, modal_y + 1),
                        SetForegroundColor(accent_gold),
                        Print("🔀 SWITCH GIT BRANCH"),
                        MoveTo(modal_x + 2, modal_y + 3),
                        SetForegroundColor(accent_green),
                        Print(format!("Target branch: {}█", branch_input)),
                        MoveTo(modal_x + 2, modal_y + 5),
                        SetForegroundColor(text_dim),
                        Print("[Enter] Switch Branch | [Esc] Cancel")
                    )?;
                }
                ActionPrompt::ExportReport { path_input } => {
                    queue!(
                        w,
                        MoveTo(modal_x + 2, modal_y + 1),
                        SetForegroundColor(accent_gold),
                        Print("📄 EXPORT MARKDOWN REPORT"),
                        MoveTo(modal_x + 2, modal_y + 3),
                        SetForegroundColor(accent_green),
                        Print(format!("Output file: {}█", path_input)),
                        MoveTo(modal_x + 2, modal_y + 5),
                        SetForegroundColor(text_dim),
                        Print("[Enter] Save Report | [Esc] Cancel")
                    )?;
                }
                ActionPrompt::CreatePullRequest {
                    repo_input,
                    title_input,
                    body_input: _,
                    head_input,
                    base_input,
                    token_input: _,
                    step,
                } => {
                    let step_desc = match step {
                        0 => format!("Repository (owner/repo): {}█", repo_input),
                        1 => format!("Pull Request Title     : {}█", title_input),
                        2 => "Description / Body     : (Press Enter to continue)".to_string(),
                        3 => format!("Source branch (head)   : {}█", head_input),
                        4 => format!("Target branch (base)   : {}█", base_input),
                        5 => "GitHub Token (optional): ******".to_string(),
                        _ => String::new(),
                    };
                    queue!(
                        w,
                        MoveTo(modal_x + 2, modal_y + 1),
                        SetForegroundColor(accent_gold),
                        Print(format!("🚀 CREATE PULL REQUEST (Step {}/6)", step + 1)),
                        MoveTo(modal_x + 2, modal_y + 3),
                        SetForegroundColor(accent_green),
                        Print(step_desc),
                        MoveTo(modal_x + 2, modal_y + 5),
                        SetForegroundColor(text_dim),
                        Print("[Enter] Next / Submit | [Esc] Cancel")
                    )?;
                }
            }
        }

        // 6. Status & Help Bar at bottom
        let status_y = start_y + height.saturating_sub(2);
        let shortcuts_y = start_y + height.saturating_sub(1);

        if let Some(ref status) = self.status_message {
            let truncated_status = truncate_to_width(status, width.saturating_sub(2) as usize);
            queue!(
                w,
                MoveTo(start_x + 1, status_y),
                SetBackgroundColor(bg_color),
                SetForegroundColor(accent_green),
                Print(&truncated_status)
            )?;
        }

        let shortcuts_text = " [Tab] Source │ [1..7] Providers │ [c] Clone │ [b] Branch │ [s] Switch │ [p] PR │ [o] Open │ [e] Export │ [a] Audit │ [Esc] Exit ";
        let rendered_shortcuts = truncate_to_width(shortcuts_text, width.saturating_sub(1) as usize);
        queue!(
            w,
            MoveTo(start_x + 1, shortcuts_y),
            SetBackgroundColor(header_bg),
            SetForegroundColor(Color::White),
            Print(&rendered_shortcuts)
        )?;

        queue!(w, ResetColor)?;
        Ok(())
    }
}

/// Helper function to truncate strings to a specific terminal column width
fn truncate_to_width(s: &str, max_width: usize) -> String {
    let mut current_width = 0;
    let mut out = String::new();
    for c in s.chars() {
        let cw = c.width().unwrap_or(0);
        if current_width + cw > max_width {
            break;
        }
        out.push(c);
        current_width += cw;
    }
    out
}

// =========================================================================
// SEARCH IMPLEMENTATIONS (GitHub, GitLab, Wikipedia, CVE, HackerNews, Audit)
// =========================================================================

#[derive(Deserialize)]
struct GitHubRepoItem {
    full_name: Option<String>,
    description: Option<String>,
    html_url: Option<String>,
    clone_url: Option<String>,
    stargazers_count: Option<u64>,
    language: Option<String>,
}

#[derive(Deserialize)]
struct GitHubSearchResponse {
    items: Option<Vec<GitHubRepoItem>>,
}

#[derive(Deserialize)]
struct GitHubIssueItem {
    title: Option<String>,
    html_url: Option<String>,
    state: Option<String>,
    body: Option<String>,
    comments: Option<u64>,
}

#[derive(Deserialize)]
struct GitHubIssueSearchResponse {
    items: Option<Vec<GitHubIssueItem>>,
}

/// Search GitHub for repositories and issues
pub fn search_github(query: &str) -> Vec<SearchResultItem> {
    if query.trim().is_empty() {
        return Vec::new();
    }
    let mut results = Vec::new();
    let client = match reqwest::blocking::Client::builder()
        .user_agent("qwx-search/0.0.3")
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(_) => return results,
    };

    // 1. Repositories
    let repo_url = format!(
        "https://api.github.com/search/repositories?q={}&per_page=10",
        urlencoding(query)
    );
    if let Ok(resp) = client.get(&repo_url).send() {
        if let Ok(data) = resp.json::<GitHubSearchResponse>() {
            if let Some(items) = data.items {
                for item in items {
                    let name = item.full_name.unwrap_or_else(|| "Unknown".to_string());
                    let desc = item.description.unwrap_or_else(|| "No description".to_string());
                    let html = item.html_url.unwrap_or_default();
                    let clone = item.clone_url.clone().or_else(|| Some(format!("{}.git", html)));
                    let stars = item.stargazers_count.unwrap_or(0);
                    let lang = item.language.unwrap_or_else(|| "N/A".to_string());

                    results.push(SearchResultItem {
                        provider: SearchProvider::GitHub,
                        title: name,
                        description: desc,
                        url: html,
                        extra_info: format!("★ {} | Lang: {}", stars, lang),
                        clone_url: clone,
                        raw_content: None,
                    });
                }
            }
        }
    }

    // 2. Issues & Pull Requests
    let issue_url = format!(
        "https://api.github.com/search/issues?q={}&per_page=5",
        urlencoding(query)
    );
    if let Ok(resp) = client.get(&issue_url).send() {
        if let Ok(data) = resp.json::<GitHubIssueSearchResponse>() {
            if let Some(items) = data.items {
                for item in items {
                    let title = item.title.unwrap_or_default();
                    let html = item.html_url.unwrap_or_default();
                    let state = item.state.unwrap_or_else(|| "open".to_string());
                    let body = item.body.unwrap_or_default();
                    let comments = item.comments.unwrap_or(0);

                    results.push(SearchResultItem {
                        provider: SearchProvider::GitHub,
                        title: format!("[Issue/PR] {}", title),
                        description: body.clone(),
                        url: html,
                        extra_info: format!("Status: {} | Comments: {}", state, comments),
                        clone_url: None,
                        raw_content: Some(body),
                    });
                }
            }
        }
    }

    results
}

#[derive(Deserialize)]
struct GitLabProjectItem {
    name_with_namespace: Option<String>,
    description: Option<String>,
    web_url: Option<String>,
    http_url_to_repo: Option<String>,
    star_count: Option<u64>,
}

/// Search GitLab for projects
pub fn search_gitlab(query: &str) -> Vec<SearchResultItem> {
    if query.trim().is_empty() {
        return Vec::new();
    }
    let mut results = Vec::new();
    let client = match reqwest::blocking::Client::builder()
        .user_agent("qwx-search/0.0.3")
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(_) => return results,
    };

    let url = format!(
        "https://gitlab.com/api/v4/projects?search={}&per_page=10",
        urlencoding(query)
    );
    if let Ok(resp) = client.get(&url).send() {
        if let Ok(items) = resp.json::<Vec<GitLabProjectItem>>() {
            for item in items {
                let title = item.name_with_namespace.unwrap_or_else(|| "Unknown".to_string());
                let desc = item.description.unwrap_or_else(|| "No description".to_string());
                let web = item.web_url.unwrap_or_default();
                let clone = item.http_url_to_repo;
                let stars = item.star_count.unwrap_or(0);

                results.push(SearchResultItem {
                    provider: SearchProvider::GitLab,
                    title,
                    description: desc,
                    url: web,
                    extra_info: format!("★ {}", stars),
                    clone_url: clone,
                    raw_content: None,
                });
            }
        }
    }

    results
}

#[derive(Deserialize)]
struct WikipediaSearchItem {
    title: Option<String>,
    snippet: Option<String>,
}

#[derive(Deserialize)]
struct WikipediaQuery {
    search: Option<Vec<WikipediaSearchItem>>,
}

#[derive(Deserialize)]
struct WikipediaResponse {
    query: Option<WikipediaQuery>,
}

/// Search Wikipedia articles
pub fn search_wikipedia(query: &str) -> Vec<SearchResultItem> {
    if query.trim().is_empty() {
        return Vec::new();
    }
    let mut results = Vec::new();
    let client = match reqwest::blocking::Client::builder()
        .user_agent("qwx-search/0.0.3")
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(_) => return results,
    };

    let url = format!(
        "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&utf8=&format=json",
        urlencoding(query)
    );
    if let Ok(resp) = client.get(&url).send() {
        if let Ok(data) = resp.json::<WikipediaResponse>() {
            if let Some(q) = data.query {
                if let Some(items) = q.search {
                    for item in items {
                        let title = item.title.unwrap_or_default();
                        let raw_snippet = item.snippet.unwrap_or_default();
                        // Strip basic html tags like <span class="searchmatch">...</span>
                        let clean_snippet = raw_snippet.replace("<span class=\"searchmatch\">", "").replace("</span>", "");
                        let article_url = format!("https://en.wikipedia.org/wiki/{}", urlencoding(&title));

                        results.push(SearchResultItem {
                            provider: SearchProvider::Wikipedia,
                            title,
                            description: clean_snippet.clone(),
                            url: article_url,
                            extra_info: "Wikipedia Article".to_string(),
                            clone_url: None,
                            raw_content: Some(clean_snippet),
                        });
                    }
                }
            }
        }
    }

    results
}

#[derive(Deserialize)]
struct NvdDescription {
    value: Option<String>,
}

#[derive(Deserialize)]
struct NvdCveData {
    id: Option<String>,
    descriptions: Option<Vec<NvdDescription>>,
}

#[derive(Deserialize)]
struct NvdVulnerability {
    cve: Option<NvdCveData>,
}

#[derive(Deserialize)]
struct NvdResponse {
    vulnerabilities: Option<Vec<NvdVulnerability>>,
}

/// Search CVE vulnerabilities (NVD / CIRCL API)
pub fn search_cve(query: &str) -> Vec<SearchResultItem> {
    if query.trim().is_empty() {
        return Vec::new();
    }
    let mut results = Vec::new();
    let client = match reqwest::blocking::Client::builder()
        .user_agent("qwx-search/0.0.3")
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(_) => return results,
    };

    let url = format!(
        "https://services.nvd.nist.gov/rest/json/cves/2.0?keywordSearch={}&resultsPerPage=10",
        urlencoding(query)
    );
    if let Ok(resp) = client.get(&url).send() {
        if let Ok(data) = resp.json::<NvdResponse>() {
            if let Some(vulns) = data.vulnerabilities {
                for v in vulns {
                    if let Some(cve) = v.cve {
                        let id = cve.id.unwrap_or_else(|| "CVE-Unknown".to_string());
                        let desc = cve
                            .descriptions
                            .and_then(|d| d.into_iter().next())
                            .and_then(|d| d.value)
                            .unwrap_or_else(|| "No details provided".to_string());
                        let cve_url = format!("https://nvd.nist.gov/vuln/detail/{}", id);

                        results.push(SearchResultItem {
                            provider: SearchProvider::Cve,
                            title: id.clone(),
                            description: desc.clone(),
                            url: cve_url,
                            extra_info: "NVD Security Advisory".to_string(),
                            clone_url: None,
                            raw_content: Some(desc),
                        });
                    }
                }
            }
        }
    }

    // Fallback: If NVD is rate-limited or returns nothing, query CIRCL CVE search
    if results.is_empty() {
        let circl_url = format!("https://cve.circl.lu/api/search/{}", urlencoding(query));
        if let Ok(resp) = client.get(&circl_url).send() {
            #[derive(Deserialize)]
            struct CirclItem {
                id: Option<String>,
                summary: Option<String>,
            }
            if let Ok(items) = resp.json::<Vec<CirclItem>>() {
                for item in items.into_iter().take(10) {
                    let id = item.id.unwrap_or_else(|| "CVE".to_string());
                    let summary = item.summary.unwrap_or_default();
                    results.push(SearchResultItem {
                        provider: SearchProvider::Cve,
                        title: id.clone(),
                        description: summary.clone(),
                        url: format!("https://cve.circl.lu/cve/{}", id),
                        extra_info: "CIRCL CVE".to_string(),
                        clone_url: None,
                        raw_content: Some(summary),
                    });
                }
            }
        }
    }

    results
}

#[derive(Deserialize)]
struct HnHit {
    title: Option<String>,
    url: Option<String>,
    points: Option<u64>,
    author: Option<String>,
    num_comments: Option<u64>,
    object_id: Option<String>,
    story_text: Option<String>,
}

#[derive(Deserialize)]
struct HnResponse {
    hits: Option<Vec<HnHit>>,
}

/// Search Hacker News (Algolia API)
pub fn search_hacker_news(query: &str) -> Vec<SearchResultItem> {
    if query.trim().is_empty() {
        return Vec::new();
    }
    let mut results = Vec::new();
    let client = match reqwest::blocking::Client::builder()
        .user_agent("qwx-search/0.0.3")
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(_) => return results,
    };

    let url = format!(
        "https://hn.algolia.com/api/v1/search?query={}&tags=story&hitsPerPage=10",
        urlencoding(query)
    );
    if let Ok(resp) = client.get(&url).send() {
        if let Ok(data) = resp.json::<HnResponse>() {
            if let Some(hits) = data.hits {
                for hit in hits {
                    let title = hit.title.unwrap_or_else(|| "No Title".to_string());
                    let points = hit.points.unwrap_or(0);
                    let comments = hit.num_comments.unwrap_or(0);
                    let author = hit.author.unwrap_or_else(|| "anonymous".to_string());
                    let id = hit.object_id.unwrap_or_default();
                    let link = hit.url.unwrap_or_else(|| {
                        format!("https://news.ycombinator.com/item?id={}", id)
                    });
                    let text = hit.story_text.unwrap_or_default();

                    results.push(SearchResultItem {
                        provider: SearchProvider::HackerNews,
                        title,
                        description: text.clone(),
                        url: link,
                        extra_info: format!("▲ {} pts | {} comments | by {}", points, comments, author),
                        clone_url: None,
                        raw_content: if text.is_empty() { None } else { Some(text) },
                    });
                }
            }
        }
    }

    results
}

/// Audit local repository / workspace dependencies for known CVEs via OSV.dev API
pub fn audit_local_workspace(current_dir: &Path) -> Vec<SearchResultItem> {
    let mut results = Vec::new();
    let client = match reqwest::blocking::Client::builder()
        .user_agent("qwx-search/0.0.3")
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => return results,
    };

    // Extract dependencies from Cargo.lock or Cargo.toml
    let mut packages: Vec<(String, String)> = Vec::new();

    let cargo_lock = current_dir.join("Cargo.lock");
    if cargo_lock.exists() {
        if let Ok(content) = fs::read_to_string(&cargo_lock) {
            let mut current_name = String::new();
            for line in content.lines() {
                let line_trim = line.trim();
                if line_trim.starts_with("name = ") {
                    current_name = line_trim
                        .trim_start_matches("name = ")
                        .trim_matches('"')
                        .to_string();
                } else if line_trim.starts_with("version = ") && !current_name.is_empty() {
                    let version = line_trim
                        .trim_start_matches("version = ")
                        .trim_matches('"')
                        .to_string();
                    packages.push((current_name.clone(), version));
                    current_name.clear();
                }
            }
        }
    }

    // If no lockfile, check Cargo.toml
    if packages.is_empty() {
        let cargo_toml = current_dir.join("Cargo.toml");
        if let Ok(content) = fs::read_to_string(&cargo_toml) {
            let mut in_deps = false;
            for line in content.lines() {
                let line_trim = line.trim();
                if line_trim.starts_with("[dependencies]") || line_trim.starts_with("[dev-dependencies]") {
                    in_deps = true;
                    continue;
                } else if line_trim.starts_with('[') {
                    in_deps = false;
                    continue;
                }
                if in_deps && line_trim.contains('=') {
                    if let Some((name, ver_part)) = line_trim.split_once('=') {
                        let name = name.trim().to_string();
                        let ver = ver_part
                            .trim()
                            .trim_matches('"')
                            .split_whitespace()
                            .next()
                            .unwrap_or("0.0.0")
                            .to_string();
                        packages.push((name, ver));
                    }
                }
            }
        }
    }

    if packages.is_empty() {
        results.push(SearchResultItem {
            provider: SearchProvider::LocalAudit,
            title: "No dependency file (Cargo.lock / Cargo.toml) found".to_string(),
            description: "Open a project directory with dependencies to run CVE audit.".to_string(),
            url: "https://osv.dev".to_string(),
            extra_info: "Local Audit".to_string(),
            clone_url: None,
            raw_content: None,
        });
        return results;
    }

    #[derive(Serialize)]
    struct OsvPackage {
        name: String,
        ecosystem: String,
    }
    #[derive(Serialize)]
    struct OsvQuery {
        version: Option<String>,
        package: OsvPackage,
    }
    #[derive(Deserialize)]
    struct OsvVuln {
        id: Option<String>,
        summary: Option<String>,
        details: Option<String>,
    }
    #[derive(Deserialize)]
    struct OsvResponse {
        vulns: Option<Vec<OsvVuln>>,
    }

    let mut audited_count = 0;
    for (pkg_name, pkg_ver) in packages.iter().take(30) {
        audited_count += 1;
        let query_body = OsvQuery {
            version: if pkg_ver.contains('{') || pkg_ver.is_empty() {
                None
            } else {
                Some(pkg_ver.clone())
            },
            package: OsvPackage {
                name: pkg_name.clone(),
                ecosystem: "crates.io".to_string(),
            },
        };

        if let Ok(resp) = client
            .post("https://api.osv.dev/v1/query")
            .json(&query_body)
            .send()
        {
            if let Ok(data) = resp.json::<OsvResponse>() {
                if let Some(vulns) = data.vulns {
                    for v in vulns {
                        let id = v.id.unwrap_or_else(|| "VULN".to_string());
                        let summary = v.summary.unwrap_or_else(|| "Vulnerability detected".to_string());
                        let details = v.details.unwrap_or_default();

                        results.push(SearchResultItem {
                            provider: SearchProvider::LocalAudit,
                            title: format!("⚠️ [{}] {} ({})", id, pkg_name, pkg_ver),
                            description: summary,
                            url: format!("https://osv.dev/vulnerability/{}", id),
                            extra_info: format!("Package: {} @ {}", pkg_name, pkg_ver),
                            clone_url: None,
                            raw_content: Some(details),
                        });
                    }
                }
            }
        }
    }

    if results.is_empty() {
        results.push(SearchResultItem {
            provider: SearchProvider::LocalAudit,
            title: format!("✓ No known vulnerability detected ({} packages analyzed)", audited_count),
            description: "All analyzed dependencies on OSV.dev appear clean with no known CVEs.".to_string(),
            url: "https://osv.dev".to_string(),
            extra_info: "Security Audit".to_string(),
            clone_url: None,
            raw_content: None,
        });
    }

    results
}

#[derive(Deserialize)]
struct DdgTopic {
    #[serde(rename = "Text")]
    text: Option<String>,
    #[serde(rename = "FirstURL")]
    first_url: Option<String>,
}

#[derive(Deserialize)]
struct DdgInstantAnswerResponse {
    #[serde(rename = "Heading")]
    heading: Option<String>,
    #[serde(rename = "AbstractText")]
    abstract_text: Option<String>,
    #[serde(rename = "AbstractURL")]
    abstract_url: Option<String>,
    #[serde(rename = "RelatedTopics")]
    related_topics: Option<Vec<DdgTopic>>,
}

/// Search DuckDuckGo (instant answer API + fallback to HTML web link results)
pub fn search_duckduckgo(query: &str) -> Vec<SearchResultItem> {
    if query.trim().is_empty() {
        return Vec::new();
    }
    let mut results = Vec::new();
    let client = match reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:120.0) Gecko/20100101 Firefox/120.0 QwxSearch/0.0.3")
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(_) => return results,
    };

    // 1. DuckDuckGo Instant Answer API
    let api_url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=0",
        urlencoding(query)
    );
    if let Ok(resp) = client.get(&api_url).send() {
        if let Ok(data) = resp.json::<DdgInstantAnswerResponse>() {
            if let Some(ref text) = data.abstract_text {
                if !text.is_empty() {
                    let heading = data.heading.unwrap_or_else(|| query.to_string());
                    let url = data.abstract_url.unwrap_or_default();
                    results.push(SearchResultItem {
                        provider: SearchProvider::Web,
                        title: heading,
                        description: text.clone(),
                        url: if url.is_empty() {
                            format!("https://html.duckduckgo.com/html/?q={}", urlencoding(query))
                        } else {
                            url
                        },
                        extra_info: "DuckDuckGo Instant Answer".to_string(),
                        clone_url: None,
                        raw_content: Some(text.clone()),
                    });
                }
            }

            if let Some(topics) = data.related_topics {
                for topic in topics.into_iter().take(10) {
                    if let (Some(text), Some(url)) = (topic.text, topic.first_url) {
                        if !text.is_empty() && !url.is_empty() {
                            let title = if let Some(dash_idx) = text.find(" - ") {
                                text[..dash_idx].to_string()
                            } else {
                                text.chars().take(50).collect()
                            };
                            results.push(SearchResultItem {
                                provider: SearchProvider::Web,
                                title,
                                description: text.clone(),
                                url,
                                extra_info: "DuckDuckGo Web".to_string(),
                                clone_url: None,
                                raw_content: Some(text),
                            });
                        }
                    }
                }
            }
        }
    }

    // 2. If no instant answers or few results, also fetch HTML / lite search
    if results.is_empty() {
        let html_url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding(query));
        if let Ok(resp) = client.get(&html_url).send() {
            if let Ok(html_text) = resp.text() {
                let engine = crate::web::HtmlReaderEngine::new(80);
                let page = engine.parse_html(&html_url, &html_text);
                for link in page.links.into_iter().take(10) {
                    if !link.url.contains("duckduckgo.com") && !link.text.trim().is_empty() {
                        results.push(SearchResultItem {
                            provider: SearchProvider::Web,
                            title: link.text.clone(),
                            description: format!("Web link: {}", link.url),
                            url: link.url,
                            extra_info: "DuckDuckGo Web".to_string(),
                            clone_url: None,
                            raw_content: None,
                        });
                    }
                }
            }
        }
    }

    // Fallback if still empty: direct DuckDuckGo link item
    if results.is_empty() {
        results.push(SearchResultItem {
            provider: SearchProvider::Web,
            title: format!("Search web for '{}'", query),
            description: "Open search results on DuckDuckGo in web reader".to_string(),
            url: format!("https://html.duckduckgo.com/html/?q={}", urlencoding(query)),
            extra_info: "DuckDuckGo Web".to_string(),
            clone_url: None,
            raw_content: None,
        });
    }

    results
}

// =========================================================================
// GIT WORKFLOW ACTIONS (Clone, Create Branch, Pull Request)
// =========================================================================

/// Clone a repository using git2 with transfer progress or fallback to git CLI
pub fn clone_repository(repo_url: &str, dest_path: &Path) -> Result<String, String> {
    clone_repository_with_progress(repo_url, dest_path, None::<fn(CloneProgress)>)
}

/// Clone repository with real-time progress reporting callback
pub fn clone_repository_with_progress<F>(
    repo_url: &str,
    dest_path: &Path,
    on_progress: Option<F>,
) -> Result<String, String>
where
    F: FnMut(CloneProgress) + 'static,
{
    use std::cell::RefCell;
    use std::rc::Rc;

    let cb_holder: Rc<RefCell<Option<F>>> = Rc::new(RefCell::new(on_progress));

    if let Some(ref mut cb) = *cb_holder.borrow_mut() {
        cb(CloneProgress {
            percentage: 5,
            indexed_objects: 0,
            total_objects: 0,
            received_bytes: 0,
            current_step: "Connecting to remote repository...".to_string(),
        });
    }

    // Attempt git2 clone with live remote callbacks
    let mut builder = git2::build::RepoBuilder::new();
    let mut callbacks = git2::RemoteCallbacks::new();

    let cb_clone = Rc::clone(&cb_holder);
    callbacks.transfer_progress(move |stats| {
        let total = stats.total_objects();
        let received = stats.received_objects();
        let indexed = stats.indexed_objects();
        let bytes = stats.received_bytes();

        let pct = if total > 0 {
            let p = ((received + indexed) as f64 / (total * 2) as f64 * 90.0) as u8;
            p.min(95).max(10)
        } else {
            20
        };

        if let Ok(mut borrow) = cb_clone.try_borrow_mut() {
            if let Some(ref mut cb) = *borrow {
                cb(CloneProgress {
                    percentage: pct,
                    indexed_objects: indexed,
                    total_objects: total,
                    received_bytes: bytes,
                    current_step: format!("Receiving objects: {}/{} - {} bytes", received, total, bytes),
                });
            }
        }
        true
    });

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);
    builder.fetch_options(fetch_options);

    match builder.clone(repo_url, dest_path) {
        Ok(_) => {
            if let Ok(mut borrow) = cb_holder.try_borrow_mut() {
                if let Some(ref mut cb) = *borrow {
                    cb(CloneProgress {
                        percentage: 100,
                        indexed_objects: 0,
                        total_objects: 0,
                        received_bytes: 0,
                        current_step: "Cloning completed successfully!".to_string(),
                    });
                }
            }
            Ok(format!(
                "Repository cloned successfully into '{}'",
                dest_path.display()
            ))
        }
        Err(err) => {
            // Fallback to git CLI command
            if let Ok(mut borrow) = cb_holder.try_borrow_mut() {
                if let Some(ref mut cb) = *borrow {
                    cb(CloneProgress {
                        percentage: 30,
                        indexed_objects: 0,
                        total_objects: 0,
                        received_bytes: 0,
                        current_step: "Falling back to Git CLI command...".to_string(),
                    });
                }
            }

            let output = Command::new("git")
                .arg("clone")
                .arg("--progress")
                .arg(repo_url)
                .arg(dest_path)
                .output()
                .map_err(|e| format!("Git execution error: {}", e))?;

            if output.status.success() {
                if let Ok(mut borrow) = cb_holder.try_borrow_mut() {
                    if let Some(ref mut cb) = *borrow {
                        cb(CloneProgress {
                            percentage: 100,
                            indexed_objects: 0,
                            total_objects: 0,
                            received_bytes: 0,
                            current_step: "Cloning completed successfully!".to_string(),
                        });
                    }
                }
                Ok(format!(
                    "Repository cloned successfully into '{}'",
                    dest_path.display()
                ))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!(
                    "Cloning failed (git2: {}, git cli: {})",
                    err,
                    stderr.trim()
                ))
            }
        }
    }
}

/// Create a new branch in the given git repository path
pub fn create_git_branch(repo_path: &Path, branch_name: &str) -> Result<String, String> {
    if branch_name.trim().is_empty() {
        return Err("Branch name cannot be empty.".to_string());
    }

    // Try git2
    if let Ok(repo) = git2::Repository::discover(repo_path) {
        if let Ok(head) = repo.head() {
            if let Ok(commit) = head.peel_to_commit() {
                if repo.branch(branch_name, &commit, false).is_ok() {
                    return Ok(format!("Branch '{}' created successfully.", branch_name));
                }
            }
        }
    }

    // Fallback to git CLI
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["checkout", "-b", branch_name.trim()])
        .output()
        .map_err(|e| format!("Git execution error: {}", e))?;

    if output.status.success() {
        Ok(format!("Branch '{}' created successfully.", branch_name))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to create branch: {}", stderr.trim()))
    }
}

/// Checkout or switch to a branch in git repository
pub fn checkout_git_branch(repo_path: &Path, branch_name: &str) -> Result<String, String> {
    if branch_name.trim().is_empty() {
        return Err("Branch name cannot be empty.".to_string());
    }

    // Use git CLI checkout
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["checkout", branch_name.trim()])
        .output()
        .map_err(|e| format!("Git execution error: {}", e))?;

    if output.status.success() {
        Ok(format!("Switched to branch '{}'.", branch_name.trim()))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to switch branch: {}", stderr.trim()))
    }
}

/// Detect origin remote owner/repo from current git workspace
pub fn detect_current_git_repo(path: &Path) -> Option<String> {
    if let Ok(repo) = git2::Repository::discover(path) {
        if let Ok(remote) = repo.find_remote("origin") {
            if let Ok(url) = remote.url() {
                let clean = url.trim_end_matches(".git");
                if let Some(pos) = clean.find("github.com/") {
                    return Some(clean[pos + "github.com/".len()..].to_string());
                } else if let Some(pos) = clean.find("github.com:") {
                    return Some(clean[pos + "github.com:".len()..].to_string());
                } else if let Some(pos) = clean.find("gitlab.com/") {
                    return Some(clean[pos + "gitlab.com/".len()..].to_string());
                } else if let Some(pos) = clean.find("gitlab.com:") {
                    return Some(clean[pos + "gitlab.com:".len()..].to_string());
                }
            }
        }
    }
    None
}

/// Export search results or CVE audit to a Markdown file
pub fn export_report_to_file(dest_path: &Path, results: &[SearchResultItem]) -> Result<String, String> {
    let mut md = String::new();
    md.push_str("# QWX Search & Security Audit Report\n\n");
    md.push_str(&format!("Total results: {}\n\n", results.len()));

    for (i, item) in results.iter().enumerate() {
        md.push_str(&format!("## {}. [{}] {}\n\n", i + 1, item.provider.name(), item.title));
        if !item.extra_info.is_empty() {
            md.push_str(&format!("- **Info:** {}\n", item.extra_info));
        }
        if !item.url.is_empty() {
            md.push_str(&format!("- **URL:** {}\n", item.url));
        }
        if let Some(ref clone_url) = item.clone_url {
            md.push_str(&format!("- **Git Clone:** `{}`\n", clone_url));
        }
        md.push_str("\n### Description / Details\n\n");
        md.push_str(&item.description);
        md.push_str("\n\n");
        if let Some(ref raw) = item.raw_content {
            md.push_str("```\n");
            md.push_str(raw);
            md.push_str("\n```\n\n");
        }
        md.push_str("---\n\n");
    }

    fs::write(dest_path, md).map_err(|e| format!("Failed to write report file: {}", e))?;
    Ok(format!("Report successfully exported to '{}'", dest_path.display()))
}

/// Open given URL in system default web browser
pub fn open_url_in_browser(url: &str) -> Result<String, String> {
    if url.trim().is_empty() {
        return Err("No URL to open.".to_string());
    }
    #[cfg(target_os = "windows")]
    let cmd = "start";
    #[cfg(target_os = "macos")]
    let cmd = "open";
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let cmd = "xdg-open";

    let mut command = Command::new(cmd);
    command.arg(url);

    match command.spawn() {
        Ok(_) => Ok(format!("Opened '{}' in default browser.", url)),
        Err(e) => Err(format!("Failed to open browser: {}", e)),
    }
}

/// Submit or create a Pull Request on GitHub
pub fn create_github_pull_request(
    repo_full_name: &str,
    title: &str,
    body: &str,
    head: &str,
    base: &str,
    token: Option<&str>,
) -> Result<String, String> {
    if repo_full_name.trim().is_empty() || title.trim().is_empty() {
        return Err("Repository name and PR title are required.".to_string());
    }

    let client = reqwest::blocking::Client::builder()
        .user_agent("qwx-search/0.0.3")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Client HTTP error: {}", e))?;

    #[derive(Serialize)]
    struct PrPayload<'a> {
        title: &'a str,
        body: &'a str,
        head: &'a str,
        base: &'a str,
    }

    let payload = PrPayload {
        title,
        body,
        head,
        base,
    };

    let api_url = format!("https://api.github.com/repos/{}/pulls", repo_full_name.trim());
    let mut req = client.post(&api_url).json(&payload);

    if let Some(tok) = token {
        if !tok.trim().is_empty() {
            req = req.header("Authorization", format!("Bearer {}", tok.trim()));
        }
    } else if let Ok(env_token) = std::env::var("GITHUB_TOKEN") {
        req = req.header("Authorization", format!("Bearer {}", env_token.trim()));
    }

    let resp = req.send().map_err(|e| format!("Network error while creating PR: {}", e))?;

    #[derive(Deserialize)]
    struct PrResponse {
        html_url: Option<String>,
    }

    let status = resp.status();
    if status.is_success() {
        if let Ok(pr_data) = resp.json::<PrResponse>() {
            let url = pr_data.html_url.unwrap_or_default();
            Ok(format!("Pull Request created successfully: {}", url))
        } else {
            Ok("Pull Request created successfully!".to_string())
        }
    } else {
        let err_msg = resp.text().unwrap_or_default();
        Err(format!("GitHub API error (HTTP {}): {}", status, err_msg))
    }
}

/// Helper simple URL encoding
fn urlencoding(s: &str) -> String {
    let mut encoded = String::new();
    for byte in s.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_provider_names() {
        assert_eq!(SearchProvider::GitHub.name(), "GitHub");
        assert_eq!(SearchProvider::Wikipedia.name(), "Wikipedia");
        assert_eq!(SearchProvider::Cve.name(), "CVE / Security");
        assert_eq!(SearchProvider::GitLab.name(), "GitLab");
        assert_eq!(SearchProvider::HackerNews.name(), "Hacker News");
        assert_eq!(SearchProvider::LocalAudit.name(), "Local Audit");
        assert_eq!(SearchProvider::Web.name(), "Web / DuckDuckGo");
        assert_eq!(SearchProvider::Web.shortcut_key(), '8');
    }

    #[test]
    fn test_search_hub_navigation() {
        let mut hub = SearchHub::new();
        assert_eq!(hub.active_provider, SearchProvider::All);
        hub.next_provider();
        assert_eq!(hub.active_provider, SearchProvider::GitHub);
        hub.prev_provider();
        assert_eq!(hub.active_provider, SearchProvider::All);
    }

    #[test]
    fn test_url_encoding() {
        assert_eq!(urlencoding("hello world"), "hello+world");
        assert_eq!(urlencoding("qwx_test-1.0"), "qwx_test-1.0");
    }

    #[test]
    fn test_truncate_to_width() {
        let text = "Hello, world!";
        assert_eq!(truncate_to_width(text, 5), "Hello");
        assert_eq!(truncate_to_width(text, 20), "Hello, world!");
    }

    #[test]
    fn test_search_hub_results_navigation() {
        let mut hub = SearchHub::new();
        hub.results.push(SearchResultItem {
            provider: SearchProvider::GitHub,
            title: "repo1".to_string(),
            description: "desc1".to_string(),
            url: "https://github.com/a/b".to_string(),
            extra_info: "".to_string(),
            clone_url: Some("https://github.com/a/b.git".to_string()),
            raw_content: None,
        });
        hub.results.push(SearchResultItem {
            provider: SearchProvider::GitHub,
            title: "repo2".to_string(),
            description: "desc2".to_string(),
            url: "https://github.com/c/d".to_string(),
            extra_info: "".to_string(),
            clone_url: Some("https://github.com/c/d.git".to_string()),
            raw_content: None,
        });

        assert_eq!(hub.selected_index, 0);
        assert_eq!(hub.selected_item().unwrap().title, "repo1");
        hub.next_result();
        assert_eq!(hub.selected_index, 1);
        assert_eq!(hub.selected_item().unwrap().title, "repo2");
        hub.prev_result();
        assert_eq!(hub.selected_index, 0);

        hub.start_clone_selected();
        assert!(matches!(hub.prompt, Some(ActionPrompt::CloneRepo { .. })));
    }

    #[test]
    fn test_search_hub_prompts() {
        let mut hub = SearchHub::new();
        hub.start_create_branch();
        assert!(matches!(hub.prompt, Some(ActionPrompt::CreateBranch { .. })));

        hub.start_create_pull_request();
        assert!(matches!(hub.prompt, Some(ActionPrompt::CreatePullRequest { .. })));
    }

    #[test]
    fn test_search_hub_drawing_smoke() {
        let hub = SearchHub::new();
        let mut buffer = Vec::new();
        let res = hub.draw(&mut buffer, 0, 0, 80, 24);
        assert!(res.is_ok());
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_search_hub_drawing_2x2_with_results_and_prompt() {
        let mut hub = SearchHub::new();
        hub.results.push(SearchResultItem {
            provider: SearchProvider::GitHub,
            title: "qwx-engine".to_string(),
            description: "Advanced text editor with 2x2 layout engine".to_string(),
            url: "https://github.com/test/qwx".to_string(),
            extra_info: "★ 320 | Lang: Rust".to_string(),
            clone_url: Some("https://github.com/test/qwx.git".to_string()),
            raw_content: Some("## Overview\nQwx terminal editor.".to_string()),
        });

        let mut buffer = Vec::new();
        let res = hub.draw(&mut buffer, 0, 0, 100, 30);
        assert!(res.is_ok());

        // Test with active clone in progress prompt
        hub.prompt = Some(ActionPrompt::CloneInProgress {
            repo_url: "https://github.com/test/qwx.git".to_string(),
            dest_path: "qwx".to_string(),
            progress_pct: 65,
            status_text: "Receiving objects: 650/1000 - 45000 bytes".to_string(),
        });
        let mut buffer2 = Vec::new();
        let res2 = hub.draw(&mut buffer2, 0, 0, 120, 35);
        assert!(res2.is_ok());
    }

    #[test]
    fn test_export_report_to_file() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("qwx_test_export_report.md");

        let items = vec![SearchResultItem {
            provider: SearchProvider::Cve,
            title: "CVE-2024-TEST".to_string(),
            description: "Test vulnerability summary".to_string(),
            url: "https://osv.dev/vulnerability/TEST".to_string(),
            extra_info: "Severity: High".to_string(),
            clone_url: None,
            raw_content: Some("Details about the vulnerability".to_string()),
        }];

        let res = export_report_to_file(&test_file, &items);
        assert!(res.is_ok());
        assert!(test_file.exists());
        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("CVE-2024-TEST"));
        assert!(content.contains("Test vulnerability summary"));
        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_empty_url_handling() {
        assert!(open_url_in_browser("").is_err());
        assert!(create_git_branch(Path::new("."), "").is_err());
        assert!(checkout_git_branch(Path::new("."), "").is_err());
    }

    #[test]
    fn test_search_hub_web_reader_integration() {
        let mut hub = SearchHub::new();
        hub.results.push(SearchResultItem {
            provider: SearchProvider::Web,
            title: "Rust Language".to_string(),
            description: "Empowering everyone to build reliable software".to_string(),
            url: "https://www.rust-lang.org".to_string(),
            extra_info: "DuckDuckGo Web".to_string(),
            clone_url: None,
            raw_content: Some("Rust is blazingly fast and memory-efficient.".to_string()),
        });

        assert!(!hub.is_viewing_web());
        hub.open_selected_in_web_reader(80);
        assert!(hub.is_viewing_web());
        assert!(hub.web_browser.current_page.is_some());

        hub.close_web_reader();
        assert!(!hub.is_viewing_web());

        hub.view_results_as_web_page(80);
        assert!(hub.is_viewing_web());
        assert!(hub.web_browser.current_page.is_some());
    }
}
