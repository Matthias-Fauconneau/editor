use {std::{str::from_utf8, fs::read, env::args}, framework::*, highlight::*};

#[throws]
fn main() {
    let file = read(args().nth(1).ok()?)?;
    let text = from_utf8(&file)?;
    fn print(TextStyle{color, style}: TextStyle, text: &str) {
        let code = match style {
            FontStyle::Normal => 31,
            FontStyle::Bold => 1,
            //_ => 31
        };
        let bgra8{b,g,r,..} = color.into();
        print!("\x1b[{}m\x1b[38;2;{};{};{}m{}\x1b(B\x1b[m",code, r,g,b, text)
    }
    for Span{text, style} in highlight(text)? {
        print(style, text);
    }
    //window(&mut Text::new(text))?
}
