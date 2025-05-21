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

    None
}

impl Block{
   pub const VALUES: [Self; 8] = [Block::LightBlue, Block::DarkBlue, Block::Orange, Block::Yellow, Block::Green, Block::Red, Block::Magenta, Block::None];
}

impl std::fmt::Display for Block{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


impl Block{
    pub fn get_string_rep(&self) -> String{
        
        use Block::*;
        format!("{}",
        match self {
            LightBlue => Colorize::cyan("[]"),
            DarkBlue => Colorize::blue("[]"),
            Orange => Colorize::yellow("[]"),//Colorize::truecolor("[]", 255, 165, 0),//140, 100, 0),
            Yellow => Colorize::truecolor("[]", 255, 210, 0),
            Green => Colorize::green("[]"),
            Red => Colorize::red("[]"),
            Magenta => Colorize::magenta("[]"),

            _ =>  Colorize::black("  ")
        })
    }
}
