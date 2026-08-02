use crossterm::style::Color;

pub const FG_DEFAULT: Color = Color::Rgb {
    r: 200,
    g: 210,
    b: 220,
};
// --- PALETTE DU FINDER ---
pub const FINDER_BORDER: Color = Color::Rgb {
    r: 45,
    g: 52,
    b: 70,
}; // Un bleu-gris sombre et discret
pub const FINDER_TEXT_MUTED: Color = Color::Rgb {
    r: 85,
    g: 95,
    b: 120,
}; // Pour les éléments inactifs/secondaires
pub const FINDER_DIR_COLOR: Color = Color::Rgb {
    r: 110,
    g: 170,
    b: 240,
}; // Un beau bleu doux pour les dossiers
pub const FINDER_FILE_COLOR: Color = Color::Rgb {
    r: 190,
    g: 200,
    b: 210,
}; // Un blanc cassé/argenté pour les fichiers
pub const FINDER_ACTIVE_SELECT: Color = Color::Rgb {
    r: 130,
    g: 110,
    b: 190,
}; // Le violet cosmique pour la sélection active
// ✨ Nouvelles couleurs pour la palette :
pub const CONSTANTS: Color = Color::Rgb {
    r: 230,
    g: 190,
    b: 100,
}; // Un Or doux, très lisible
pub const NAMESPACES: Color = Color::Rgb {
    r: 150,
    g: 180,
    b: 200,
}; // Un gris-bleu pour les modules
pub const DIRECTIVES: Color = Color::Rgb {
    r: 200,
    g: 120,
    b: 180,
}; // Un rose/magenta pastel
// Ton fameux "Bleu Noir" pour les commentaires (discret mais lisible)
pub const COMMENTS: Color = Color::Rgb {
    r: 50,
    g: 70,
    b: 100,
};

// Un bleu clair/cyan très pur pour les types
pub const TYPES: Color = Color::Rgb {
    r: 100,
    g: 170,
    b: 255,
};

// Un bleu électrique pastel pour les fonctions, qui reste dans ton thème
pub const FUNCTIONS: Color = Color::Rgb {
    r: 80,
    g: 200,
    b: 240,
};

// Pour contraster un peu sans casser l'ambiance froide :
pub const KEYWORDS: Color = Color::Rgb {
    r: 180,
    g: 150,
    b: 255,
}; // Violet doux
pub const STRINGS: Color = Color::Rgb {
    r: 120,
    g: 200,
    b: 160,
}; // Vert d'eau/Menthe
pub const NUMBERS: Color = Color::Rgb {
    r: 240,
    g: 160,
    b: 120,
};

// --- PALETTE DE L'INTERFACE ---
// Une teinte abyssale pour les bordures inactives, pour qu'elles se fondent dans le décor
pub const UI_BORDER_INACTIVE: Color = Color::Rgb {
    r: 35,
    g: 40,
    b: 55,
};

// Un violet cosmique très doux pour mettre en valeur le panneau actif sans agresser l'œil
pub const UI_BORDER_ACTIVE: Color = Color::Rgb {
    r: 130,
    g: 110,
    b: 190,
};

// Un gris-bleu fantôme pour les textes d'information (pourcentages, numéro de vue)
pub const UI_TEXT_MUTED: Color = Color::Rgb {
    r: 90,
    g: 100,
    b: 120,
};

// Pour la barre Dmenu : un fond sombre et un texte clair, fini le vert fluo !
pub const UI_DMENU_BG: Color = Color::Rgb {
    r: 45,
    g: 50,
    b: 70,
};
pub const UI_DMENU_FG: Color = Color::Rgb {
    r: 220,
    g: 225,
    b: 240,
};

pub fn get_color_for_capture(capture_name: &str) -> Color {
    match capture_name {
        "constant" | "constant.builtin" | "constant.character.escape" => CONSTANTS,

        "namespace" => NAMESPACES,

        "attribute" | "keyword.directive" => DIRECTIVES,
        "comment" | "line_comment" | "block_comment" => COMMENTS,

        "type" | "primitive_type" | "type.builtin" => TYPES,

        "function" | "function.macro" | "function.method" | "method" => FUNCTIONS,

        "keyword" | "keyword.function" | "keyword.return" | "keyword.operator" => KEYWORDS,

        "string" | "string_literal" | "character" => STRINGS,

        "number" | "integer" | "float" | "boolean" => NUMBERS,

        "operator" | "punctuation.bracket" | "punctuation.delimiter" => FG_DEFAULT,

        "variable" | "variable.parameter" | "variable.builtin" | "property" => FG_DEFAULT,

        _ => FG_DEFAULT,
    }
}
