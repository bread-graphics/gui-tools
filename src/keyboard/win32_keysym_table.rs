// MIT/Apache2 License

#![cfg(windows)]

use super::KeyType::{self, *};

const UN: KeyType = Unknown;

// table of win32 virtual keycodes to beetle keycodes
pub const WIN32_KEYSYM_TABLE: [KeyType; 0xA4] = [
    // 0x00-0x07 are of no interest to us
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    BackSpace, // VK_BACK = 0x08
    Tab,       // VK_TAB = 0x09
    // 0x0A-0B are reserved
    UN,
    UN,
    Clear, // VK_CLEAR = 0x0C
    Enter, // VK_RETURN = 0x0D
    // 0x0E-0F are reserved
    UN,
    UN,
    LeftShift,   // VK_SHIFT = 0x10
    LeftControl, // VK_CONTROL = 0x11
    LeftAlt,     // VK_MENU = 0x12
    Pause,       // VK_PAUSE = 0x13
    CapsLock,    // VK_CAPITAL = 0x14
    Kana,        // VK_KANA = 0x15
    // 0x16-1A are IME keys we don't care about
    UN,
    UN,
    UN,
    UN,
    UN,
    Escape, // VK_ESCAPE = 0x1B
    // 0x1C-1F are IME keys we don't care about
    UN,
    UN,
    UN,
    UN,
    Space,       // VK_SPACE = 0x20
    PageUp,      // VK_PRIOR = 0x21
    PageDown,    // VK_NEXT = 0x22
    End,         // VK_END = 0x23
    Home,        // VK_HOME = 0x24
    Left,        // VK_LEFT = 0x25
    Up,          // VK_UP = 0x26
    Right,       // VK_RIGHT = 0x27
    Down,        // VK_DOWN = 0x28
    UN,          // VK_SELECT = 0x29, don't care
    UN,          // VK_PRINT = 0x2A, don't care
    UN,          // VK_EXECUTE = 0x2B, don't care
    PrintScreen, // VK_SNAPSHOT = 0x2C
    Insert,      // VK_INSERT = 0x2D
    Delete,      // VK_DELETE = 0x2E
    Help,        // VK_HELP = 0x2F
    // number keys,
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
    // 0x3A-40 are undefined
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
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
    // two windows keys
    Windows, // VK_LWIN = 0x5B
    Windows, // VK_RWIN = 0x5C
    UN,      // VK_APPS = 0x5D, don't care
    UN,      // 0x5E is reserved
    UN,      // VK_SLEEP = 0x5F, don't care
    // the numpad 0-9
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    Multiply,  // VK_MULTIPLY = 0x6A
    Add,       // VK_ADD = 0x6B
    Separator, // VK_SEPARATOR = 0x6C
    Subtract,  // VK_SUBTRACT = 0x6D
    Decimal,   // VK_DECIMAL = 0x6E
    Divide,    // VK_DIVIDE = 0x6F
    // the function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    // 0x88-8F are undefined
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    UN,
    NumLock,    // VK_NUMLOCK = 0x90
    ScrollLock, // VK_SCROLL = 0x91
    // 0x92-9F are either undefined or we don't care
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
    // left and right shift
    LeftShift,   // VK_LSHIFT = 0xA0
    RightShift,  // VK_RSHIFT = 0xA1
    LeftControl, // VK_LCONTROL = 0xA2
    RightControl, // VK_RCONTROL = 0xA3
                 // rest are function keys we don't support
];
