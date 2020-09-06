// MIT/Apache2 License

#![allow(clippy::enum_glob_use)] // i think clippy would understand

use super::KeyType::{self, *};

#[allow(non_upper_case_globals)]
const UN: KeyType = Unknown;

// table of x11 keysyms to beetle keycodes
pub const X11_KEYSYM_TABLE: [KeyType; 0xAF] = [
    // the first 0x1F numbers are unused
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    Space,            // 0x20 = XK_Space
    ExclamationMark,  // 0x21 = XK_exclam
    DoubleQuote,      // 0x22 = XK_quotedbl
    NumberSign,       // 0x23 = XK_numbersign
    Dollar,           // 0x24 = XK_dollar
    Percent,          // 0x25 = XK_percent
    Ampersand,        // 0x26 = XK_ampersand,
    Quote,            // 0x27 = XK_apostrophe
    LeftParenthesis,  // 0x28 = XK_parenleft
    RightParenthesis, // 0x29 = XK_parenright
    Asterisk,         // 0x2a = XK_asterisk
    Plus,             // 0x2b = XK_plus
    Comma,            // 0x2c = XK_comma,
    Minus,            // 0x2d = XK_minus
    Period,           // 0x2e = XK_period,
    Slash,            // 0x2f = XK_Slash,
    // number keys
    N0,
    N1,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    Colon,        // 0x3a = XK_colon
    Semicolon,    // 0x3b = XK_semicolon
    Less,         // 0x3c = XK_less
    Equals,       // 0x3d = XK_equal
    Greater,      // 0x3e = XK_greater,
    QuestionMark, // 0x3f = XK_question,
    At,           // 0x40 = XK_at,
    // the alphabet
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBracket,  // 0x5b = XK_bracketleft,
    BackSlash,    // 0x5c = XK_backslash
    RightBracket, // 0x5d = XK_bracketright
    Circumflex,   // 0x5e = XK_asciicircum
    Underscore,   // 0x5f = XK_underscore
    BackQuote,    // 0x60 = XK_grave
    // the alphabet, again
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBrace,  // 0x7b = XK_braceleft
    Bar,        // 0x7c = XK_bar
    RightBrace, // 0x7d = XK_braceright
    Tilde,      // 0x7e = XK_tidle
    // 0x7f to 0x9f are unused
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    Space,                   // 0xa0 = XK_nobreakspace
    InvertedExclamationMark, // 0xa1 = XK_exclamdown
    // TODO: this block contains characters that should be filled out
    // such as shift, caps lock, etc
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    //        Hyphen, // 0xad = XK_hyphen
];
