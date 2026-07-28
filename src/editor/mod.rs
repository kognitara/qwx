use crate::editor::theme::get_color_for_capture;
use ropey::Rope;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;
use std::path::PathBuf;
use tree_sitter::{InputEdit, Language, Point, QueryCursor};
use tree_sitter::{Parser, Tree};
use tree_sitter::{Query, StreamingIterator};
use tree_sitter_highlight::HighlightConfiguration;
pub mod theme;
/// Helper pour instancier et configurer proprement HighlightConfiguration
fn create_config(
    scope: &str,
    lang: Language,
    query: &'static str,
    theme_keys: &[&'static str],
) -> Option<LangConfig> {
    let mut ts_config = HighlightConfiguration::new(lang, scope, query, "", "").ok()?;
    ts_config.configure(theme_keys);
    Some(LangConfig {
        ts_config,
        query_string: query,
    })
}

/// Associe une extension de fichier à sa configuration Tree-sitter correspondante.
fn detect_langage(extension: &str, theme_keys: &[&'static str]) -> Option<LangConfig> {
    match extension {
        "ada" | "adb" => create_config(
            "ada",
            Language::from(tree_sitter_ada::LANGUAGE),
            "",
            theme_keys,
        ),
        "agda" => create_config(
            "agda",
            Language::from(tree_sitter_agda::LANGUAGE),
            tree_sitter_agda::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "asm" | "s" => create_config(
            "asm",
            Language::from(tree_sitter_asm::LANGUAGE),
            tree_sitter_asm::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "sh" | "bash" => create_config(
            "bash",
            Language::from(tree_sitter_bash::LANGUAGE),
            tree_sitter_bash::HIGHLIGHT_QUERY,
            theme_keys,
        ),
        "bat" | "cmd" => create_config(
            "batch",
            Language::from(tree_sitter_batch::LANGUAGE),
            tree_sitter_batch::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "c" | "h" => create_config(
            "c",
            Language::from(tree_sitter_c::LANGUAGE),
            tree_sitter_c::HIGHLIGHT_QUERY,
            theme_keys,
        ),
        "cs" => create_config(
            "c_sharp",
            Language::from(tree_sitter_c_sharp::LANGUAGE),
            tree_sitter_c_sharp::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "cmake" => create_config(
            "cmake",
            Language::from(tree_sitter_cmake::LANGUAGE),
            tree_sitter_cmake::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "cpp" | "cc" | "cxx" | "hpp" => create_config(
            "cpp",
            Language::from(tree_sitter_cpp::LANGUAGE),
            tree_sitter_cpp::HIGHLIGHT_QUERY,
            theme_keys,
        ),
        "css" => create_config(
            "css",
            Language::from(tree_sitter_css::LANGUAGE),
            tree_sitter_css::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "d" => create_config("d", Language::from(tree_sitter_d::LANGUAGE), "", theme_keys),
        "dart" => create_config(
            "dart",
            Language::from(tree_sitter_dart::LANGUAGE),
            tree_sitter_dart::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "diff" | "patch" => create_config(
            "diff",
            Language::from(tree_sitter_diff::LANGUAGE),
            "",
            theme_keys,
        ),
        "ex" | "exs" => create_config(
            "elixir",
            Language::from(tree_sitter_elixir::LANGUAGE),
            tree_sitter_elixir::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "elm" => create_config(
            "elm",
            Language::from(tree_sitter_elm::LANGUAGE),
            tree_sitter_elm::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "erl" | "hrl" => create_config(
            "erlang",
            Language::from(tree_sitter_erlang::LANGUAGE),
            tree_sitter_erlang::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "fish" => create_config(
            "fish",
            tree_sitter_fish::language(),
            tree_sitter_fish::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "fs" | "fsi" | "fsx" => create_config(
            "fsharp",
            Language::from(tree_sitter_fsharp::LANGUAGE_FSHARP),
            tree_sitter_fsharp::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "glsl" | "vert" | "frag" => create_config(
            "glsl",
            Language::from(tree_sitter_glsl::LANGUAGE_GLSL),
            "",
            theme_keys,
        ),
        "go" => create_config(
            "go",
            Language::from(tree_sitter_go::LANGUAGE),
            tree_sitter_go::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "gql" | "graphql" => create_config(
            "graphql",
            Language::from(tree_sitter_graphql::LANGUAGE),
            "",
            theme_keys,
        ),
        "hs" => create_config(
            "haskell",
            Language::from(tree_sitter_haskell::LANGUAGE),
            tree_sitter_haskell::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "hcl" | "tf" => create_config(
            "hcl",
            Language::from(tree_sitter_hcl::LANGUAGE),
            "",
            theme_keys,
        ),
        "hlsl" => create_config(
            "hlsl",
            Language::from(tree_sitter_hlsl::LANGUAGE_HLSL),
            "",
            theme_keys,
        ),
        "html" | "htm" => create_config(
            "html",
            Language::from(tree_sitter_html::LANGUAGE),
            tree_sitter_html::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "ini" => create_config(
            "ini",
            Language::from(tree_sitter_ini::LANGUAGE),
            tree_sitter_ini::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "java" => create_config(
            "java",
            Language::from(tree_sitter_java::LANGUAGE),
            tree_sitter_java::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "js" | "mjs" | "cjs" => create_config(
            "javascript",
            Language::from(tree_sitter_javascript::LANGUAGE),
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            theme_keys,
        ),
        "json" => create_config(
            "json",
            Language::from(tree_sitter_json::LANGUAGE),
            tree_sitter_json::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "lua" => create_config(
            "lua",
            Language::from(tree_sitter_lua::LANGUAGE),
            tree_sitter_lua::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "make" | "makefile" | "Makefile" => create_config(
            "make",
            Language::from(tree_sitter_make::LANGUAGE),
            tree_sitter_make::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "nix" => create_config(
            "nix",
            Language::from(tree_sitter_nix::LANGUAGE),
            tree_sitter_nix::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "m" => create_config(
            "objc",
            Language::from(tree_sitter_objc::LANGUAGE),
            tree_sitter_objc::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "odin" => create_config(
            "odin",
            Language::from(tree_sitter_odin::LANGUAGE),
            tree_sitter_odin::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "pl" | "pm" => create_config(
            "perl",
            Language::from(tree_sitter_perl::LANGUAGE),
            "",
            theme_keys,
        ),
        "php" => create_config(
            "php",
            Language::from(tree_sitter_php::LANGUAGE_PHP),
            tree_sitter_php::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "py" | "pyw" => create_config(
            "python",
            Language::from(tree_sitter_python::LANGUAGE),
            tree_sitter_python::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "r" => create_config(
            "r",
            Language::from(tree_sitter_r::LANGUAGE),
            tree_sitter_r::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "rb" => create_config(
            "ruby",
            Language::from(tree_sitter_ruby::LANGUAGE),
            tree_sitter_ruby::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "rs" => create_config(
            "rust",
            Language::from(tree_sitter_rust::LANGUAGE),
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "scala" | "sc" => create_config(
            "scala",
            Language::from(tree_sitter_scala::LANGUAGE),
            tree_sitter_scala::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "swift" => create_config(
            "swift",
            Language::from(tree_sitter_swift::LANGUAGE),
            tree_sitter_swift::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "ts" | "mts" | "cts" => create_config(
            "typescript",
            Language::from(tree_sitter_typescript::LANGUAGE_TYPESCRIPT),
            tree_sitter_typescript::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "xml" | "xsd" => create_config(
            "xml",
            Language::from(tree_sitter_xml::LANGUAGE_XML),
            tree_sitter_xml::XML_HIGHLIGHT_QUERY,
            theme_keys,
        ),
        "yaml" | "yml" => create_config(
            "yaml",
            Language::from(tree_sitter_yaml::LANGUAGE),
            tree_sitter_yaml::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "zig" => create_config(
            "zig",
            Language::from(tree_sitter_zig::LANGUAGE),
            tree_sitter_zig::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        _ => None, // Extension inconnue
    }
}

/// Représente la configuration de coloration pour un langage spécifique
pub struct LangConfig {
    pub ts_config: HighlightConfiguration,
    pub query_string: &'static str,
}
#[derive(Default)]
pub struct Ji {
    pub rope: Rope,
    pub file_path: Option<PathBuf>,
    pub query: Option<Query>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub parser: Parser,
    pub syntax_tree: Option<Tree>,
    pub lang_config: Option<LangConfig>,
}

impl Ji {
    /// Supprime le caractère situé sous le curseur (Touche Suppr)
    pub fn delete(&mut self) {
        // 1. Calculer l'index absolu du curseur
        let cursor_char_idx = self.rope.line_to_char(self.cursor_line) + self.cursor_col;

        // Si on est à la toute fin du fichier, il n'y a rien à supprimer
        if cursor_char_idx >= self.rope.len_chars() {
            return;
        }

        // 2. Identifier le caractère ciblé (exactement sous le curseur)
        let target_char = self.rope.char(cursor_char_idx);
        let char_len_bytes = target_char.len_utf8();
        let byte_idx = self.rope.char_to_byte(cursor_char_idx);

        // 3. Déterminer les positions graphiques pour Tree-sitter
        let start_point = Point::new(self.cursor_line, self.cursor_col);

        let mut old_end_point = start_point;
        if target_char == '\n' {
            old_end_point.row += 1;
            old_end_point.column = 0;
        } else {
            old_end_point.column += char_len_bytes;
        }

        // 4. Notifier l'arbre syntaxique de la suppression
        if let Some(ref mut tree) = self.syntax_tree {
            let edit = InputEdit {
                start_byte: byte_idx,
                old_end_byte: byte_idx + char_len_bytes,
                new_end_byte: byte_idx,
                start_position: start_point,
                old_end_position: old_end_point,
                new_end_position: start_point, // Le curseur ne bouge pas
            };
            tree.edit(&edit);
        }

        // 5. Supprimer le caractère dans la structure Rope
        self.rope.remove(cursor_char_idx..(cursor_char_idx + 1));

        // 6. Mettre à jour l'arbre syntaxique
        self.update_syntax_tree();
    }
    /// Sauvegarde le contenu de l'éditeur dans le fichier d'origine
    pub fn save(&mut self) -> io::Result<()> {
        if let Some(ref path) = self.file_path {
            // Création ou écrasement du fichier
            let file = File::create(path)?;

            // Utilisation d'un BufWriter pour une écriture disque performante
            let writer = std::io::BufWriter::new(file);

            // Ropey possède une méthode hyper optimisée pour s'écrire dans un flux
            self.rope.write_to(writer)?;
        }
        Ok(())
    }
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref)?;
        let rope = Rope::from_reader(BufReader::new(file))?;

        // 1. Votre catalogue de tokens utilisé par votre gestionnaire de thème
        let theme_keys = vec![
            "keyword",
            "keyword.function",
            "keyword.return",
            "keyword.operator",
            "function",
            "function.macro",
            "function.method",
            "method",
            "string",
            "string_literal",
            "character",
            "number",
            "integer",
            "float",
            "boolean",
            "comment",
            "line_comment",
            "block_comment",
            "type",
            "primitive_type",
            "type.builtin",
            "operator",
            "punctuation.bracket",
            "punctuation.delimiter",
            "variable",
            "variable.parameter",
            "variable.builtin",
            "property",
            "attribute",
            "label",
            "constant",
            "constant.builtin",
            "constant.character.escape",
            "namespace",
            "keyword.directive",
            "punctuation.special",
        ];

        if let Some(extension) = path_ref.extension().and_then(|ext| ext.to_str())
            && let Some(config) = detect_langage(&extension.to_lowercase(), &theme_keys)
        {
            // On prépare la Query proprement avec les règles SCM transportées par config
            let query_obj = Query::new(&config.ts_config.language, config.query_string).ok();

            // ✨ CORRECTION ICI : On instancie le parseur ET on lui donne le langage !
            let mut parser = Parser::new();
            // Note : Si ton compilateur râle à cause du "&", retire-le simplement.
            let _ = parser.set_language(&config.ts_config.language);
            let mut ji = Self {
                rope,
                file_path: Some(path_ref.to_path_buf()),
                cursor_line: 0,
                cursor_col: 0,
                parser,
                syntax_tree: None,
                lang_config: Some(config),
                query: query_obj, // C'est parfait !
            };

            if ji.lang_config.is_some() {
                ji.update_syntax_tree();
            }
            return Ok(ji);
        }
        Ok(Ji::default())
    }

    /// Insère un caractère à la position actuelle du curseur (ligne, col)
    pub fn insert_char(&mut self, ch: char) {
        // 1. Calculer l'index absolu en caractères et en octets (bytes)
        let char_idx = self.rope.line_to_char(self.cursor_line) + self.cursor_col;
        let byte_idx = self.rope.char_to_byte(char_idx);

        // 2. Définir les coordonnées graphiques de départ
        let start_point = Point::new(self.cursor_line, self.cursor_col);

        // 3. Calculer les nouvelles coordonnées graphiques après l'insertion
        let mut new_end_point = start_point;
        if ch == '\n' {
            new_end_point.row += 1;
            new_end_point.column = 0;
        } else {
            new_end_point.column += ch.len_utf8();
        }

        // 4. Notifier l'arbre syntaxique du changement (si un arbre existe)
        if let Some(ref mut tree) = self.syntax_tree {
            let edit = InputEdit {
                start_byte: byte_idx,
                old_end_byte: byte_idx,
                new_end_byte: byte_idx + ch.len_utf8(),
                start_position: start_point,
                old_end_position: start_point,
                new_end_position: new_end_point,
            };
            tree.edit(&edit); // Ajuste les index de l'arbre de manière chirurgicale
        }

        // 5. Insérer réellement le caractère dans la Rope
        self.rope.insert_char(char_idx, ch);

        // 6. Mettre à jour la position du curseur
        if ch == '\n' {
            self.cursor_line += 1;
            self.cursor_col = 0;
        } else {
            self.cursor_col += 1;
        }

        // 7. Relancer le parsing incrémental ultra-rapide
        self.update_syntax_tree();
    }

    /// Supprime le caractère situé juste avant le curseur (Retour arrière)
    pub fn backspace(&mut self) {
        // Si on est tout au début du fichier, on ne peut rien supprimer
        if self.cursor_line == 0 && self.cursor_col == 0 {
            return;
        }

        // 1. Déterminer la position du caractère à supprimer (juste avant le curseur)
        let cursor_char_idx = self.rope.line_to_char(self.cursor_line) + self.cursor_col;
        let target_char_idx = cursor_char_idx - 1;

        let target_char = self.rope.char(target_char_idx);
        let char_len_bytes = target_char.len_utf8();
        let byte_idx = self.rope.char_to_byte(target_char_idx);

        // 2. Déterminer les anciennes et nouvelles positions du curseur graphique
        let old_end_point = Point::new(self.cursor_line, self.cursor_col);
        let mut start_point = old_end_point;

        if target_char == '\n' {
            // Si on supprime un retour à la ligne, le curseur remonte à la ligne précédente
            start_point.row -= 1;
            // On se place à la fin de cette ligne précédente (avant la fusion des lignes)
            start_point.column = self.rope.line(start_point.row).len_chars() - 1;
        } else {
            start_point.column -= 1;
        }

        // 3. Notifier l'arbre syntaxique de la suppression
        if let Some(ref mut tree) = self.syntax_tree {
            let edit = InputEdit {
                start_byte: byte_idx,
                old_end_byte: byte_idx + char_len_bytes,
                new_end_byte: byte_idx,
                start_position: start_point,
                old_end_position: old_end_point,
                new_end_position: start_point,
            };
            tree.edit(&edit);
        }

        // 4. Supprimer le caractère dans la Rope
        self.rope.remove(target_char_idx..cursor_char_idx);

        // 5. Déplacer le curseur physique vers sa nouvelle position
        self.cursor_line = start_point.row;
        self.cursor_col = start_point.column;

        // 6. Mettre à jour l'arbre syntaxique
        self.update_syntax_tree();
    }

    pub fn update_syntax_tree(&mut self) {
        // Si aucun langage n'est configuré, impossible de générer un arbre.
        if self.lang_config.is_none() {
            return;
        }

        // On crée une référence locale à la Rope pour la closure
        let rope = &self.rope;

        // Appel de parse_with_options (ou parse) en utilisant le parseur interne de Ji
        // tree-sitter demande des morceaux d'octets au fur et à mesure de ses besoins.
        let tree = self.parser.parse_with_options(
            &mut |byte_offset, _position| {
                if byte_offset < rope.len_bytes() {
                    // Ropey trouve instantanément le bloc de texte ("chunk") contenant cet octet
                    let (chunk, chunk_byte_idx, _, _) = rope.chunk_at_byte(byte_offset);
                    // On renvoie la tranche exacte d'octets demandée par le parseur
                    &chunk.as_bytes()[byte_offset - chunk_byte_idx..]
                } else {
                    // Fin du texte atteinte, on renvoie une tranche vide
                    &[] as &[u8]
                }
            },
            self.syntax_tree.as_ref(), // Fournit l'ancien arbre pour permettre le calcul incrémental
            None,                      // Pas d'options de parsing spécifiques nécessaires
        );

        // On sauvegarde le nouvel arbre mis à jour
        self.syntax_tree = tree;
    }
    /// Retourne une liste de segments textuels (String) associés à leur couleur Crossterm,
    /// couvrant l'intégralité du document de manière continue.
    pub fn get_colored_spans(&self) -> Vec<(String, crossterm::style::Color)> {
        let mut spans = Vec::new();
        let total_bytes = self.rope.len_bytes();
        if total_bytes == 0 {
            return spans;
        }

        // 1. Récupérer toutes les captures brutes
        let mut raw_highlights = Vec::new();
        if let (Some(tree), Some(query)) = (&self.syntax_tree, &self.query) {
            let mut cursor = QueryCursor::new();
            let text_bytes = self.rope.to_string().into_bytes();
            let mut matches = cursor.matches(query, tree.root_node(), text_bytes.as_slice());

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    let start = capture.node.start_byte();
                    let end = capture.node.end_byte();
                    let name = &query.capture_names()[capture.index as usize];
                    raw_highlights.push((start, end, name.to_string()));
                }
            }
        }

        // 2. Trier les captures : par début croissant, puis par fin décroissante (les plus larges d'abord)
        raw_highlights.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));

        // 3. Linéariser les captures pour éviter les chevauchements
        let mut current_byte = 0;
        let text_string = self.rope.to_string();
        let text_bytes = text_string.as_bytes();

        for (start, end, name) in raw_highlights {
            // Ignorer les captures obsolètes ou déjà dépassées (imbriquées)
            if start < current_byte {
                continue;
            }

            // S'il y a un trou entre la position actuelle et le début de la capture,
            // on ajoute du texte avec la couleur par défaut.
            if start > current_byte {
                if let Ok(text_slice) = std::str::from_utf8(&text_bytes[current_byte..start]) {
                    spans.push((text_slice.to_string(), theme::FG_DEFAULT));
                }
                current_byte = start;
            }

            // Ajouter la zone colorée
            if let Ok(text_slice) = std::str::from_utf8(&text_bytes[start..end]) {
                let color = get_color_for_capture(&name);
                spans.push((text_slice.to_string(), color));
                current_byte = end;
            }
        }

        // Ajouter le reste du fichier s'il reste du texte non coloré à la fin
        if current_byte < total_bytes
            && let Ok(text_slice) = std::str::from_utf8(&text_bytes[current_byte..total_bytes])
        {
            spans.push((text_slice.to_string(), theme::FG_DEFAULT));
        }
        spans
    }
}
