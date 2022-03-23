pub fn is_whitespace(c: char) -> bool {
    c == '\x20'     // Space
    || c == '\x09'  // Horizontal tab
    || c == '\x0A'  // Line feed or New Line
    || c == '\x0D' // Carriage return
}

pub fn is_unescaped(c: char) -> bool {
    ('\x20'..='\x21').contains(&c) || ('\x23'..='\x5B').contains(&c) || c >= '\x5D'
}

// TODO: fix about uXXXX
pub fn is_escape_target(c: char) -> bool {
    let escape_targets = [
        '\x22', '\x5C', '\x2F', '\x62', '\x66', '\x6E', '\x72', '\x74',
    ];
    escape_targets.contains(&c)
}

// TODO: fix about uXXXX
pub fn escape(c: char) -> Option<char> {
    match c {
        '\x22' => Some('\u{0022}'), // "    quotation mark  U+0022
        '\x5C' => Some('\u{005C}'), // \    reverse solidus U+005C
        '\x2F' => Some('\u{002F}'), // /    solidus         U+002F
        '\x62' => Some('\u{0008}'), // b    backspace       U+0008
        '\x66' => Some('\u{000C}'), // f    form feed       U+000C
        '\x6E' => Some('\u{000A}'), // n    line feed       U+000A
        '\x72' => Some('\u{000D}'), // r    carriage return U+000D
        '\x74' => Some('\u{0009}'), // t    tab             U+0009
        // uXXXX                U+XXXX
        _ => None,
    }
}
