use colored::Colorize;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Block {
    LightBlue,
    DarkBlue,
    Orange,
    Yellow,
    Green,
    Red,
    Magenta,

    None,
}

impl Block {
    ///All the values in Block except for None
    pub const VALUES: [Self; 7] = [
        Block::LightBlue,
        Block::DarkBlue,
        Block::Orange,
        Block::Yellow,
        Block::Green,
        Block::Red,
        Block::Magenta,
    ];
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Block {
    pub fn get_string_rep(&self, use_color: bool) -> String {
        if use_color {
            use Block::*;
            format!(
                "{}",
                match self {
                    LightBlue => Colorize::cyan("[]"),
                    DarkBlue => Colorize::blue("[]"),
                    Orange => Colorize::yellow("[]"),
                    Yellow => Colorize::bright_yellow("[]"),
                    Green => Colorize::green("[]"),
                    Red => Colorize::red("[]"),
                    Magenta => Colorize::magenta("[]"),

                    _ => Colorize::black("  "),
                }
            )
        } else {
            String::from(if *self == Block::None { "  " } else { "[]" })
        }
    }
}
