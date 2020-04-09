use {std::{str::from_utf8, fs::read, env::args}, framework::*, highlight::*};

#[throws]
fn main() {
    let file = read(args().nth(1).ok()?)?;
    let text = from_utf8(&file)?;
    fn set_color(bgra8{b,g,r,..}: bgra8) { print!("\x1b[38;2;{};{};{}m",r,g,b) }
    for Span{text, style:TextStyle{color, ..}} in highlight(text)? {
        set_color(color.into());
        print!("{}", text);
    }
    //window(&mut Text::new(text))?
}
