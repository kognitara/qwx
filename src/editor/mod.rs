use ropey::Rope;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};

pub struct Ji {
    pub rope: Rope,
    pub cursor_idx: usize, // Position absolue du curseur dans le texte
    pub file_path: Option<String>,
}

impl Ji {
    /// Crée un document vide
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            cursor_idx: 0,
            file_path: None,
        }
    }
    /// Open the path in the editor
    pub fn open(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let rope = Rope::from_reader(BufReader::new(file))?;

        Ok(Self {
            rope,
            cursor_idx: 0,
            file_path: Some(path.to_string()),
        })
    }

    /// Insère un caractère à la position actuelle du curseur
    pub fn insert_char(&mut self, ch: char) {
        self.rope.insert_char(self.cursor_idx, ch);
        self.cursor_idx += 1; // On avance le curseur après l'insertion
    }

    /// Supprime le caractère juste avant le curseur (Backspace)
    pub fn backspace(&mut self) {
        if self.cursor_idx > 0 {
            self.rope.remove((self.cursor_idx - 1)..self.cursor_idx);
            self.cursor_idx -= 1;
        }
    }

    /// Supprime le caractère sous le curseur (Delete)
    pub fn delete(&mut self) {
        if self.cursor_idx < self.rope.len_chars() {
            self.rope.remove(self.cursor_idx..(self.cursor_idx + 1));
        }
    }

    /// Sauvegarde le document sur le disque
    pub fn save(&self) -> io::Result<()> {
        if let Some(ref path) = self.file_path {
            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);
            self.rope.write_to(&mut writer)?;
        }
        Ok(())
    }
}
