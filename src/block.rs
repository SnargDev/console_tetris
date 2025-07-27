#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Block {
    LightBlue,
    DarkBlue,
    Orange,
    Yellow,
    Green,
    Red,
    Magenta,

    Void,
}

impl Block {
    ///All the values in Block except for Void
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
    pub fn get_string_rep_colored(&self) -> String {
        use Block::*;
        use crossterm::style::Stylize;
        match *self {
            LightBlue => "[]".cyan(),
            DarkBlue => "[]".blue(),
            Orange => "[]".yellow(),
            Yellow => "[]".dark_yellow(),
            Green => "[]".green(),
            Red => "[]".red(),
            Magenta => "[]".magenta(),
            Void => " .".grey(),
        }
        .to_string()
    }
}
