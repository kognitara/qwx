use crossterm::style::{Color, ResetColor};
use crossterm::{
    cursor::MoveTo,
    style::{Print, SetForegroundColor},
    terminal::{ClearType, size},
};
use crossterm::{queue, terminal::Clear};
use std::{
    io::{Result, Write},
    path::Path,
};
use unicode_width::UnicodeWidthChar;
use walkdir::WalkDir;

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
    /// │ ROOTS FILES           │ PREVIEW                  │
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

pub struct Finder {
    layout: FinderLayout,
    directories: Vec<String>,
    sub_directories: Vec<String>,
    files: Vec<String>,
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

    pub fn filter(&mut self, path: &Path, research: String) -> (Vec<String>, Vec<String>) {
        let files = list_files(path);
        let dirs = list_dirs(path);
        let sub_dirs = list_sub_dirs(path); // <-- Ajout ici pour recharger la liste complète

        let research_lower = research.to_lowercase();

        self.files = files
            .into_iter()
            .filter(|x| x.to_lowercase().starts_with(&research_lower))
            .collect();

        self.directories = dirs
            .into_iter()
            .filter(|x| x.to_lowercase().starts_with(&research_lower))
            .collect();

        // Filtrage sur la liste fraîche, pas sur l'ancienne !
        self.sub_directories = sub_dirs
            .into_iter()
            .filter(|x| x.to_lowercase().starts_with(&research_lower))
            .collect();

        (self.get_directories(), self.get_files())
    }
    pub fn draw<W: Write>(&self, w: &mut W, research: String) -> Result<()> {
        queue!(w, Clear(ClearType::All))?;
        if self.layout == FinderLayout::Grid {
            let search_placeholder = if research.is_empty() {
                "Type to search"
            } else {
                research.as_str()
            };

            // MODIFICATION ICI : On décale la grille d'un caractère vers la droite
            let mid_x = (self.width / 2) + 1;

            let header_h = 3; // Lignes 0, 1, 2 allouées à RESEARCH
            let footer_h = 5; // Hauteur allouée aux blocs de statistiques en bas

            // Le point central de l'axe Y pour les 4 quadrants principaux
            let main_y_start = header_h;
            let main_h = self.height.saturating_sub(header_h + footer_h);
            let mid_y = main_y_start + (main_h / 2);

            // --- CALCUL DES DIMENSIONS ---
            // Attention : Supprime ou commente le deuxième `let mid_x = self.width / 2;`
            // qui se trouvait un peu plus bas dans ton code pour ne pas écraser ta modification !

            let research_x = self.width.saturating_sub(search_placeholder.len() as u16) / 2;

            queue!(
                w,
                MoveTo(0, 0),
                SetForegroundColor(Color::DarkGrey),
                Print(format!(
                    "┌{}┐",
                    "─".repeat((self.width.saturating_sub(2)) as usize)
                )),
                MoveTo(0, 1),
                Print("│"),
                MoveTo(research_x, 1),
                Print(search_placeholder),
                MoveTo(self.width - 1, 1),
                Print("│"),
                MoveTo(0, 2),
                Print(format!(
                    "├{}┤",
                    "─".repeat((self.width.saturating_sub(2)) as usize)
                )),
                MoveTo(mid_x, 2),
                Print("┬")
            )?;

            // ==========================================
            // 2. ZONE PRINCIPALE : BORDURES DES QUADRANTS
            // ==========================================
            let right_x = self.width.saturating_sub(1);

            // Ligne de séparation horizontale centrale (on démarre à 1 et on s'arrête avant la fin pour les bordures)
            for x in 1..right_x {
                queue!(
                    w,
                    MoveTo(x, mid_y),
                    SetForegroundColor(Color::DarkGrey),
                    Print("─")
                )?;
            }

            // Lignes verticales (Gauche, Centre, Droite)
            for y in main_y_start..self.height.saturating_sub(footer_h) {
                queue!(
                    w,
                    SetForegroundColor(Color::DarkGrey),
                    MoveTo(0, y),
                    Print("│"), // Bordure gauche
                    MoveTo(mid_x, y),
                    Print("│"), // Ligne centrale
                    MoveTo(right_x, y),
                    Print("│") // Bordure droite
                )?;
            }
            // Intersections de la ligne centrale horizontale
            queue!(
                w,
                SetForegroundColor(Color::DarkGrey),
                MoveTo(0, mid_y),
                Print("├"), // Intersection gauche
                MoveTo(mid_x, mid_y),
                Print("┼"), // Croix centrale
                MoveTo(right_x, mid_y),
                Print("┤") // Intersection droite
            )?;

            // Titres des sections
            let max_files_display = (self.height.saturating_sub(footer_h) - mid_y - 2) as usize;
            let max_dirs_display = (mid_y - main_y_start - 2) as usize;

            // 1. On restaure TA largeur d'origine pour un padding parfait jusqu'à la bordure !
            let pane_width = (mid_x.saturating_sub(5)) as usize;

            // NOUVELLE FONCTION SÉCURISÉE : Largeur EXACTE et nettoyage des caractères
            let format_padded = |text: &str, total_w: usize| -> String {
                // Sécurité absolue : on retire les tabulations et retours à la ligne qui cassent le terminal
                let safe_text = text
                    .replace("\t", "    ")
                    .replace("\n", "")
                    .replace("\r", "");

                let max_text_w = total_w.saturating_sub(1); // On garde 1 place pour l'espace initial

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
                        MoveTo(4, main_y_start + 2 + i as u16),
                        SetForegroundColor(Color::Cyan),
                        Print(padded_name),
                        ResetColor
                    )?;
                } else {
                    queue!(
                        w,
                        MoveTo(4, main_y_start + 2 + i as u16),
                        ResetColor,
                        SetForegroundColor(Color::Blue),
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
                    // SÉLECTION ÉPURÉE pour les fichiers sélectionnés
                    queue!(
                        w,
                        MoveTo(4, mid_y + 2 + i as u16),
                        SetForegroundColor(Color::White),
                        Print(padded_name),
                        ResetColor
                    )?;
                } else {
                    queue!(
                        w,
                        MoveTo(4, mid_y + 2 + i as u16),
                        ResetColor,
                        SetForegroundColor(Color::DarkGrey),
                        Print(padded_name)
                    )?;
                }
            }

            // ==========================================
            // AFFICHAGE DES SOUS-DOSSIERS
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
                queue!(
                    w,
                    MoveTo(right_pane_x, main_y_start + 2 + i as u16),
                    ResetColor, // <-- CORRECTION ICI
                    SetForegroundColor(Color::Cyan),
                    Print(padded_name)
                )?;
            }
            // ==========================================
            // 4. ZONE FOOTER : STATISTIQUES ET RÉSULTATS
            // ==========================================
            let footer_y = self.height.saturating_sub(footer_h);
            let bottom_y = self.height.saturating_sub(1);

            queue!(w, SetForegroundColor(Color::DarkGrey))?;

            // Lignes horizontales du footer (en évitant les coins)
            for x in 1..right_x {
                queue!(
                    w,
                    MoveTo(x, footer_y),
                    SetForegroundColor(Color::DarkGrey),
                    Print("─")
                )?; // Séparateur supérieur
                queue!(
                    w,
                    MoveTo(x, footer_y + 2),
                    SetForegroundColor(Color::DarkGrey),
                    Print("─")
                )?; // Séparateur du milieu
                queue!(
                    w,
                    MoveTo(x, bottom_y),
                    SetForegroundColor(Color::DarkGrey),
                    Print("─")
                )?; // Ligne de fermeture en bas
            }

            // Lignes verticales du footer
            for y in footer_y..bottom_y {
                queue!(
                    w,
                    SetForegroundColor(Color::DarkGrey),
                    MoveTo(0, y),
                    Print("│"), // Bordure gauche
                    MoveTo(mid_x, y),
                    Print("│"), // Colonne centrale
                    MoveTo(right_x, y),
                    Print("│") // Bordure droite
                )?;
            }

            // Toutes les intersections du footer pour une grille parfaite
            queue!(
                w,
                SetForegroundColor(Color::DarkGrey),
                MoveTo(0, footer_y),
                Print("├"),
                MoveTo(mid_x, footer_y),
                Print("┼"),
                MoveTo(right_x, footer_y),
                Print("┤"),
                // Ligne du milieu du footer
                MoveTo(0, footer_y + 2),
                Print("├"),
                MoveTo(mid_x, footer_y + 2),
                Print("┼"),
                MoveTo(right_x, footer_y + 2),
                Print("┤"),
                // Ligne du bas (Coins inférieurs)
                MoveTo(0, bottom_y),
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
                MoveTo(2, footer_y + 1),
                SetForegroundColor(Color::DarkGrey),
                Print(dir_count_str),
                MoveTo(mid_x + 2, footer_y + 1),
                Print(sub_dir_count_str), // <-- ON UTILISE LA VARIABLE ICI
                MoveTo(2, footer_y + 3),
                Print(" SUB ROOT FILES FOUND : 0 "), // (Prêt pour quand tu feras les sous-fichiers !)
                MoveTo(mid_x + 2, footer_y + 3),
                Print(file_count_str)
            )?;
        }
        // On s'assure de réinitialiser les couleurs après avoir dessiné le Finder
        queue!(w, ResetColor)?;
        Ok(())
    }
}
