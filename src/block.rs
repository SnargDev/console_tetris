use colored::Colorize;

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone)]
pub struct Color(u8, u8, u8);


impl Block{
    pub fn get_string_rep(&self) -> String{
        let colors = [
            Color(50,50,180),
            Color(0,0,255),
            Color(128, 30, 0),
            Color(128, 128, 0),
            Color(0,255,0),
            Color(255, 0, 0),
            Color(128, 0, 128),
            Color(0,0,0)
        ];

        let c = colors[*self as usize];
        format!("{}", Colorize::truecolor("[]", c.0, c.1, c.2))
    }
}
