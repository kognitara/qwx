use crossterm::queue;
use crossterm::style::ResetColor;
use crossterm::{
    cursor::MoveTo,
    style::{Print, SetForegroundColor},
    terminal::size,
};
use std::{
    io::{Result, Write},
    path::Path,
};
use unicode_width::UnicodeWidthChar;
use walkdir::WalkDir;

use crate::editor::theme::{
    FINDER_ACTIVE_SELECT, FINDER_BORDER, FINDER_DIR_COLOR, FINDER_FILE_COLOR, FINDER_TEXT_MUTED,
};

#[derive(Clone, PartialEq, Eq)]
pub enum FinderLayout {
    ///
    /// ┌──────────────────────────────────────────────────┐
    /// │                   RESEARCH                       │
    /// └──────────────────────────────────────────────────┘
    /// ┌───────────────────────┬──────────────────────────┐
    /// │ ROOT DIRECTORIES      │ SUB ROOTS DIRECTORIES    │
    /// │                       │                          │
    /// │                       │                          │
    /// │                       │                          │
    /// ├───────────────────────┼──────────────────────────┤
    /// │ ROOTS FILES           │ SUB ROOTS FILES          │
    /// │                       │                          │
    /// │                       │                          │
    /// │                       │                          │
    /// └───────────────────────┴──────────────────────────┘
    /// ┌───────────────────────┬──────────────────────────┐
    /// │ ROOT DIRS FOUNDED     │ SUB ROOTS DIRS FOUNDED   │
    /// ├───────────────────────┼──────────────────────────┤
    /// │ SUB ROOT DIRS FOUNDED │ SUB ROOT FILES FOUNDED   │
    /// └───────────────────────┴──────────────────────────┘
    ///
    Grid,
    ///
    /// ┌─────────────────────────────────────────────────┐
    /// │                   RESEARCH                      │
    /// └─────────────────────────────────────────────────┘
    /// ┌───────────────────────┬─────────────────────────┐
    /// │ ROOT DIRECTORIES      │ ROOTS FILES             │
    /// │                       │                         │
    /// │                       │                         │
    /// │                       │                         │
    /// │                       │                         │
    /// ├───────────────────────┼─────────────────────────┤
    /// │ SUB ROOT DIRECTORIES  │ PREVIEW                 │
    /// │                       │                         │
    /// │                       │                         │
    /// │                       │                         │
    /// │                       │                         │
    /// └───────────────────────┴─────────────────────────┘
    /// ┌───────────────────────┬─────────────────────────┐
    /// │ ROOT DIRS FOUNDED     │ ROOT FILES FOUNDED      │
    /// ├───────────────────────┼─────────────────────────┤
    /// │ SUB ROOT DIRS FOUNDED │ SUB ROOT FILES FOUNDED  │
    /// └───────────────────────┴─────────────────────────┘
    ///
    GridSecondary,
    ///
    /// ┌─────────────────────────────────────────────────┐
    /// │                   RESEARCH                      │
    /// └─────────────────────────────────────────────────┘
    /// ┌───────────────────────┬─────────────────────────┐
    /// │ DIRS                  │ FILES                   │
    /// │                       │                         │
    /// │                       │                         │
    /// │                       │                         │
    /// │                       │                         │
    /// └───────────────────────┴─────────────────────────┘
    /// ┌───────────────────────┬─────────────────────────┐
    /// │ DIRS FOUNDED          │ FILES FOUNDED           │
    /// └───────────────────────┴─────────────────────────┘
    ///
    SideBySide,
    /// ┌─────────────────────────────────────────────────┐
    /// │                   RESEARCH                      │
    /// └─────────────────────────────────────────────────┘
    /// ┌───────────────┬─────────────────┬───────────────┐
    /// │ PARENT DIRS   │ ACTIVE DIR      │ CHILD DIRS    │
    /// │               │                 │               │
    /// │               │                 │               │
    /// │               │                 │               │
    /// │               │                 │               │
    /// ├───────────────┴─────────────────┴───────────────┤
    /// │                    FILES                        │
    /// │                                                 │
    /// │                                                 │
    /// │                                                 │
    /// │                                                 │
    /// │                                                 │
    /// │                                                 │
    /// └─────────────────────────────────────────────────┘
    /// ┌───────────────┬─────────────────┬───────────────┐
    /// │ PARENT FOUNDED│ DIRS FOUNDED    │ CHILD FOUNDED │
    /// └───────────────┴─────────────────┴───────────────┘
    ///
    Miller,
    /// ┌─────────────────────────────────────────────────┐
    /// │                   RESEARCH                      │
    /// └─────────────────────────────────────────────────┘
    /// ┌─────────────────────────────────────────────────┐
    /// │                  DIRECTORIES                    │
    /// │                                                 │
    /// │                                                 │
    /// └─────────────────────────────────────────────────┘
    /// ┌──────────────────────┬──────────────────────────┐
    /// │                      │                          │
    /// │ CURRENT DIRECTORY    │ CURRENT FILES            │
    /// │                      │                          │
    /// │                      │                          │
    /// │                      │                          │
    /// │                      │                          │
    /// │                      │                          │
    /// └──────────────────────┴──────────────────────────┘
    /// ┌──────────────────────┬──────────────────────────┐
    /// │ DIRS FOUNDED         │ FILES FOUNDED            │
    /// └──────────────────────┴──────────────────────────┘
    ///
    Commander,
    ///
    /// ┌──────────────┬──────┬──────┐
    /// │              │ src/ │ app/ │
    /// │    Root /    ├──────┼──────┤
    /// │              │ doc/ │ lib/ │
    /// └──────────────┴──────┴──────┘
    ///
    Mosaic,
}

impl FinderLayout {
    // Fonction pour cycler vers le layout suivant
    pub fn next(&self) -> Self {
        match self {
            Self::Grid => Self::GridSecondary,
            Self::GridSecondary => Self::SideBySide,
            Self::SideBySide => Self::Miller,
            Self::Miller => Self::Commander,
            Self::Commander => Self::Mosaic,
            Self::Mosaic => Self::Grid,
        }
    }
    pub fn previous(&self) -> Self {
        match self {
            Self::Grid => Self::Mosaic,
            Self::GridSecondary => Self::Grid,
            Self::SideBySide => Self::GridSecondary,
            Self::Miller => Self::SideBySide,
            Self::Commander => Self::Miller,
            Self::Mosaic => Self::Commander,
        }
    }
}

pub fn list_files(path: &Path) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();
    for entry in WalkDir::new(path)
        .max_depth(1)
        .min_depth(1)
        .into_iter()
        .flatten()
    {
        if entry.path().is_file() {
            // On extrait uniquement le nom du fichier (ex: "evaluate.c")
            if let Some(name) = entry.file_name().to_str() {
                files.push(name.to_string());
            }
        }
    }
    files
}

pub fn list_dirs(path: &Path) -> Vec<String> {
    let mut dirs: Vec<String> = Vec::new();
    for entry in WalkDir::new(path)
        .max_depth(1)
        .min_depth(1)
        .into_iter()
        .flatten()
    {
        if entry.path().is_dir() {
            // On extrait uniquement le nom du dossier
            if let Some(name) = entry.file_name().to_str() {
                dirs.push(name.to_string());
            }
        }
    }
    dirs
}

pub fn list_sub_dirs(path: &Path) -> Vec<String> {
    let mut dirs: Vec<String> = Vec::new();
    for entry in WalkDir::new(path)
        .max_depth(2)
        .min_depth(2)
        .into_iter()
        .flatten()
    {
        if entry.path().is_dir() {
            // Pour les sous-dossiers, on garde le chemin relatif propre (ex: "parent/enfant")
            if let Ok(rel_path) = entry.path().strip_prefix(path) {
                dirs.push(rel_path.to_string_lossy().to_string());
            }
        }
    }
    dirs
}

pub fn list_sub_files(path: &Path) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();
    for entry in WalkDir::new(path)
        .max_depth(2)
        .min_depth(2)
        .into_iter()
        .flatten()
    {
        if entry.path().is_file() {
            // Pour les sous-dossiers, on garde le chemin relatif propre (ex: "parent/enfant")
            if let Ok(rel_path) = entry.path().strip_prefix(path) {
                files.push(rel_path.to_string_lossy().to_string());
            }
        }
    }
    files
}

pub struct Finder {
    layout: FinderLayout,
    directories: Vec<String>,
    sub_directories: Vec<String>,
    sub_files: Vec<String>,
    files: Vec<String>,
    pub selected_dir: usize,
    pub selected_sub_dir: usize,
    pub selected_sub_file: usize,
    width: u16,
    height: u16,
}

impl Finder {
    #[must_use]
    pub fn new(path: &Path, layout: FinderLayout) -> Self {
        let (w, h) = size().unwrap_or((80, 100));
        Self {
            layout,
            directories: list_dirs(path),
            files: list_files(path),
            sub_directories: list_sub_dirs(path),
            sub_files: list_sub_files(path),
            selected_dir: 0,
            selected_sub_dir: 0,
            selected_sub_file: 0,
            width: w,
            height: h,
        }
    }
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }
    pub fn get_directories(&self) -> Vec<String> {
        self.directories.to_vec()
    }
    pub fn get_files(&self) -> Vec<String> {
        self.files.to_vec()
    }
    pub fn next_dir(&mut self) {
        if !self.directories.is_empty() {
            self.selected_dir = (self.selected_dir + 1) % self.directories.len();
        }
    }

    pub fn prev_dir(&mut self) {
        if !self.directories.is_empty() {
            self.selected_dir = if self.selected_dir > 0 {
                self.selected_dir - 1
            } else {
                self.directories.len() - 1
            };
        }
    }
    pub fn filter(&mut self, path: &Path, research: String) -> (Vec<String>, Vec<String>) {
        let files = list_files(path);
        let dirs = list_dirs(path);
        let sub_dirs = list_sub_dirs(path); // <-- Ajout ici pour recharger la liste complète

        let research_lower = research.to_lowercase();

        self.files = files
            .into_iter()
            .filter(|x| x.to_lowercase().contains(&research_lower))
            .collect();

        self.directories = dirs
            .into_iter()
            .filter(|x| x.to_lowercase().contains(&research_lower))
            .collect();

        // Filtrage sur la liste fraîche, pas sur l'ancienne !
        self.sub_directories = sub_dirs
            .into_iter()
            .filter(|x| x.to_lowercase().contains(&research_lower))
            .collect();

        (self.get_directories(), self.get_files())
    }
    pub fn draw<W: Write>(
        &self,
        w: &mut W,
        research: String,
        start_x: u16, // Marge gauche
        start_y: u16, // Marge haute (souvent 0)
        width: u16,   // Largeur restreinte (ex: 180 max)
        height: u16,  // Hauteur totale
    ) -> Result<()> {
        // ==========================================
        // CORRECTION 1 : NETTOYAGE DU FOND
        // ==========================================
        // On remplit la zone exacte du Finder avec des espaces.
        // Cela crée un "fond opaque" qui masque le code de l'éditeur
        // situé en dessous, sans faire clignoter le reste du terminal.
        let empty_line = " ".repeat(width as usize);
        for y in start_y..(start_y + height) {
            queue!(w, MoveTo(start_x, y), Print(&empty_line))?;
        }

        if self.layout == FinderLayout::Grid {
            let search_placeholder = if research.is_empty() {
                "Type to search"
            } else {
                research.as_str()
            };

            // ==========================================
            // CORRECTION 2 : ALIGNEMENT DU MILIEU
            // ==========================================
            // On a retiré le `+ 1`. La ligne centrale du Finder va
            // maintenant s'emboîter parfaitement avec celle de l'App.
            let mid_x = start_x + (width / 2);
            let right_x = start_x + width.saturating_sub(1);

            let header_h = 3;
            let footer_h = 5;

            let main_y_start = start_y + header_h;
            let main_h = height.saturating_sub(header_h + footer_h);
            let mid_y = main_y_start + (main_h / 2);

            let research_x = start_x + (width.saturating_sub(search_placeholder.len() as u16) / 2);

            // ==========================================
            // 1. DESSIN DE L'EN-TÊTE (RESEARCH)
            // ==========================================
            queue!(
                w,
                MoveTo(start_x, start_y),
                SetForegroundColor(FINDER_BORDER),
                Print(format!(
                    "┌{}┐",
                    "─".repeat((width.saturating_sub(2)) as usize)
                )),
                MoveTo(start_x, start_y + 1),
                Print("│"),
                MoveTo(research_x, start_y + 1),
                SetForegroundColor(FINDER_TEXT_MUTED),
                Print(search_placeholder),
                SetForegroundColor(FINDER_BORDER),
                MoveTo(right_x, start_y + 1),
                Print("│"),
                MoveTo(start_x, start_y + 2),
                Print(format!(
                    "├{}┤",
                    "─".repeat((width.saturating_sub(2)) as usize)
                )),
                MoveTo(mid_x, start_y + 2),
                Print("┬")
            )?;

            // ==========================================
            // 2. ZONE PRINCIPALE : BORDURES DES QUADRANTS
            // ==========================================

            // Ligne de séparation horizontale centrale
            for x in (start_x + 1)..right_x {
                queue!(
                    w,
                    MoveTo(x, mid_y),
                    SetForegroundColor(FINDER_BORDER),
                    Print("─")
                )?;
            }

            // Lignes verticales (Gauche, Centre, Droite)
            for y in main_y_start..(start_y + height.saturating_sub(footer_h)) {
                queue!(
                    w,
                    SetForegroundColor(FINDER_BORDER),
                    MoveTo(start_x, y),
                    Print("│"),
                    MoveTo(mid_x, y),
                    Print("│"),
                    MoveTo(right_x, y),
                    Print("│")
                )?;
            }

            // Intersections de la ligne centrale horizontale
            queue!(
                w,
                SetForegroundColor(FINDER_BORDER),
                MoveTo(start_x, mid_y),
                Print("├"),
                MoveTo(mid_x, mid_y),
                Print("┼"),
                MoveTo(right_x, mid_y),
                Print("┤")
            )?;

            // Titres et calcul des dimensions
            let max_files_display =
                (start_y + height.saturating_sub(footer_h) - mid_y - 2) as usize;
            let max_dirs_display = (mid_y - main_y_start - 2) as usize;

            // Ajustement de la largeur de la zone en prenant en compte l'offset X
            let pane_width = (mid_x.saturating_sub(start_x + 5)) as usize;

            let format_padded = |text: &str, total_w: usize| -> String {
                let safe_text = text.replace('\t', "    ").replace(['\n', '\t'], "");

                let max_text_w = total_w.saturating_sub(1);

                let mut acc = 0;
                let mut truncated = String::new();
                for c in safe_text.chars() {
                    let cw = c.width().unwrap_or(0);
                    if acc + cw > max_text_w {
                        break;
                    }
                    truncated.push(c);
                    acc += cw;
                }
                truncated
            };

            // ==========================================
            // AFFICHAGE DES DOSSIERS
            // ==========================================
            for (i, dir) in self.directories.iter().take(max_dirs_display).enumerate() {
                let padded_name = format_padded(dir, pane_width);
                if i == 0 {
                    queue!(
                        w,
                        MoveTo(start_x + 4, main_y_start + 2 + i as u16),
                        SetForegroundColor(FINDER_ACTIVE_SELECT), // Met en valeur le premier/actif en violet cosmique
                        Print(padded_name),
                        ResetColor
                    )?;
                } else {
                    queue!(
                        w,
                        MoveTo(start_x + 4, main_y_start + 2 + i as u16),
                        ResetColor,
                        SetForegroundColor(FINDER_DIR_COLOR), // Le bleu doux pour les dossiers
                        Print(padded_name)
                    )?;
                }
            }

            // ==========================================
            // AFFICHAGE DES FICHIERS
            // ==========================================
            for (i, file) in self.files.iter().take(max_files_display).enumerate() {
                let padded_name = format_padded(file, pane_width);
                if i == 0 {
                    queue!(
                        w,
                        MoveTo(start_x + 4, mid_y + 2 + i as u16),
                        SetForegroundColor(FINDER_ACTIVE_SELECT),
                        Print(padded_name),
                        ResetColor
                    )?;
                } else {
                    queue!(
                        w,
                        MoveTo(start_x + 4, mid_y + 2 + i as u16),
                        ResetColor,
                        SetForegroundColor(FINDER_FILE_COLOR), // Le blanc argenté pour les fichiers standards
                        Print(padded_name)
                    )?;
                }
            }

            // ==========================================
            // AFFICHAGE DES SOUS-DOSSIERS (Haut Droite)
            // ==========================================
            let max_sub_dirs_display = (mid_y - main_y_start - 2) as usize;
            let right_pane_x = mid_x + 4;

            for (i, sub_dir) in self
                .sub_directories
                .iter()
                .take(max_sub_dirs_display)
                .enumerate()
            {
                let padded_name = format_padded(sub_dir, pane_width);
                if i == self.selected_sub_dir {
                    queue!(
                        w,
                        MoveTo(right_pane_x, main_y_start + 2 + i as u16),
                        SetForegroundColor(FINDER_ACTIVE_SELECT),
                        Print(padded_name),
                        ResetColor
                    )?;
                } else {
                    queue!(
                        w,
                        MoveTo(right_pane_x, main_y_start + 2 + i as u16),
                        ResetColor,
                        SetForegroundColor(FINDER_TEXT_MUTED),
                        Print(padded_name)
                    )?;
                }
            }

            // ==========================================
            // AFFICHAGE DES SOUS-FICHIERS (Bas Droite)
            // ==========================================
            let max_sub_files_display =
                (start_y + height.saturating_sub(footer_h) - mid_y - 2) as usize;

            for (i, sub_file) in self
                .sub_files
                .iter()
                .take(max_sub_files_display)
                .enumerate()
            {
                let padded_name = format_padded(sub_file, pane_width);
                if i == self.selected_sub_file {
                    queue!(
                        w,
                        MoveTo(right_pane_x, mid_y + 2 + i as u16),
                        SetForegroundColor(FINDER_FILE_COLOR),
                        Print(padded_name),
                        ResetColor
                    )?;
                } else {
                    queue!(
                        w,
                        MoveTo(right_pane_x, mid_y + 2 + i as u16),
                        ResetColor,
                        SetForegroundColor(FINDER_TEXT_MUTED),
                        Print(padded_name)
                    )?;
                }
            }

            // ==========================================
            // 4. ZONE FOOTER : STATISTIQUES ET RÉSULTATS
            // ==========================================
            let footer_y = start_y + height.saturating_sub(footer_h);
            let bottom_y = start_y + height.saturating_sub(1);

            // Lignes horizontales du footer (en évitant les coins)
            for x in (start_x + 1)..right_x {
                queue!(
                    w,
                    MoveTo(x, footer_y),
                    SetForegroundColor(FINDER_BORDER),
                    Print("─")
                )?;
                queue!(
                    w,
                    MoveTo(x, footer_y + 2),
                    SetForegroundColor(FINDER_BORDER),
                    Print("─")
                )?;
                queue!(
                    w,
                    MoveTo(x, bottom_y),
                    SetForegroundColor(FINDER_BORDER),
                    Print("─")
                )?;
            }

            // Lignes verticales du footer
            for y in footer_y..bottom_y {
                queue!(
                    w,
                    SetForegroundColor(FINDER_BORDER),
                    MoveTo(start_x, y),
                    Print("│"),
                    MoveTo(mid_x, y),
                    Print("│"),
                    MoveTo(right_x, y),
                    Print("│")
                )?;
            }

            // Toutes les intersections du footer pour une grille parfaite
            queue!(
                w,
                SetForegroundColor(FINDER_BORDER),
                MoveTo(start_x, footer_y),
                Print("├"),
                MoveTo(mid_x, footer_y),
                Print("┼"),
                MoveTo(right_x, footer_y),
                Print("┤"),
                MoveTo(start_x, footer_y + 2),
                Print("├"),
                MoveTo(mid_x, footer_y + 2),
                Print("┼"),
                MoveTo(right_x, footer_y + 2),
                Print("┤"),
                MoveTo(start_x, bottom_y),
                Print("└"),
                MoveTo(mid_x, bottom_y),
                Print("┴"),
                MoveTo(right_x, bottom_y),
                Print("┘")
            )?;

            // Affichage des compteurs dynamiques
            let dir_count_str = format!(" ROOT DIRS FOUND : {} ", self.directories.len());
            let file_count_str = format!(" ROOT FILES FOUND : {} ", self.files.len());
            let sub_dir_count_str =
                format!(" SUB ROOTS DIRS FOUND : {} ", self.sub_directories.len());

            queue!(
                w,
                MoveTo(start_x + 2, footer_y + 1),
                SetForegroundColor(FINDER_TEXT_MUTED),
                Print(dir_count_str),
                MoveTo(mid_x + 2, footer_y + 1),
                Print(sub_dir_count_str),
                MoveTo(start_x + 2, footer_y + 3),
                Print(" SUB ROOT FILES FOUND : 0 "),
                MoveTo(mid_x + 2, footer_y + 3),
                Print(file_count_str)
            )?;
        }

        queue!(w, ResetColor)?;
        Ok(())
    }
}
