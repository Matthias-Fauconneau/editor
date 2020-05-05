use {std::sync::Arc, framework::{Error,throws, text::{Style,Attribute, TextSize,TextRange,Color,FontStyle}}};

pub struct StyledText { pub text: Arc<String>, pub style: Vec<Attribute<Style>> }
#[cfg(feature="rust-analyzer")] mod highlight {
    use {super::*, rust_analyzer::*};
    #[throws]
    pub fn style(highlight: impl Iterator<Item=HighlightedRange>) -> impl Iterator<Item=StyledTextRange> {
        highlight.complete().map(|HighlightedRange{range, highlight, ..}| {
            use HighlightTag::*;
            StyledTextRange{
                range,
                style: TextStyle{
                    color: match highlight.tag {
                        Module => bgr{b:0.,r:1.,g:1./3.},
                        Keyword if !highlight.modifiers.iter().any(|it| it == HighlightModifier::ControlFlow) => bgr{b:2./3.,r:2./3.,g:2./3.},
                        Function|Macro => bgr{b:2./3.,r:1.,g:2./3.},
                        Struct|TypeAlias|BuiltinType|TypeParam|Enum => bgr{b:2./3,r:0.,g:2./3.},
                        Field => bgr{b:0.,r:0.,g:2./3},
                        Trait => bgr{b:1.,r:1./2.,g:1.},
                        StringLiteral|NumericLiteral|EnumVariant => bgr{b:0.,r:1.,g:1./3.},
                        Lifetime|Attribute => bgr{b:1.,r:1./3.,g:1./3.},
                        Comment => bgr{b:1./2.,r:1./2.,g:1./2.},
                        _ => bgr{b:1.,r:1.,g:1.},
                    },
                    style:
                        if highlight.modifiers.iter().any(|it| it == HighlightModifier::ControlFlow) { FontStyle::Bold } //else { FontStyle::Normal }
                        else {
                            match highlight.tag {
                                Keyword => FontStyle::Bold, // fixme: Italic
                                _ => FontStyle::Normal
                            }
                        }
                }
            }
        })
    }
    pub fn highlight() -> StyledText { let TextHighlight{text, highlight} = highlight()?; StyledText{text, style: style(highlight().into_iter()).collect()} }
}
#[cfg(not(feature="rust-analyzer"))] mod highlight { // Stub highlight to develop text editor while rust-analyzer is too slow, blocked on parallel items: rust-analyzer#3485,3720
    use super::*;
    #[allow(dead_code)] // ?
    #[throws]
    pub fn highlight() -> StyledText {
        let file = std::fs::read("src/main.rs")?;
        let source = std::str::from_utf8(&file)?;
        let mut depth = 0;
        let mut last_root_bracket = None;
        let mut target = String::with_capacity(source.len());
        for (offset, char) in source.char_indices() {
            if char == '{' {
                if depth == 0 { last_root_bracket = Some(offset); }
                depth += 1;
            }
            if depth == 0 { target.push(char) }
            if char == '\n' { last_root_bracket = None; }
            if char == '}' {
                depth -= 1;
                if let Some(backtrack) = last_root_bracket { target.push_str(&source[backtrack..=offset]) }
            }
        }
        let text = Arc::new(target);
        let style = vec![Attribute::<Style>{range: TextRange::up_to(TextSize::of(text.as_str())), attribute: Style{ color: Color{b:1.,r:1.,g:1.}, style: FontStyle::Normal }}];
        StyledText{text, style}
    }
}

#[cfg(feature="iced")]
mod iced {
    pub use iced::{Settings, Sandbox, window};
    use iced::{Element, text_input, TextInput};

    #[derive(Default)]
    pub struct Editor {
        text: String,
        text_input_state: text_input::State,
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        InputChanged(String),
    }

    impl Sandbox for Editor {
        type Message = Message;
        fn new() -> Self {
            let highlight = super::highlight::highlight().unwrap();
            Self{
                text: highlight.text.to_string(),
                ..Editor::default()
            }
        }
        fn title(&self) -> String { String::from("Editor") }
        fn update(&mut self, message: Message) {
            match message {
                Message::InputChanged(value) => self.text = value,
            }
        }
        fn view(&mut self) -> Element<Message> {
            TextInput::new(&mut self.text_input_state, "", &self.text, Message::InputChanged).into()
        }
    }
}

#[throws]
fn main() {
    #[cfg(feature="env_logger")] env_logger::init();
    #[cfg(feature="rstack-self")] framework::rstack_self()?;
    #[cfg(feature="signal-hook")] framework::signal_hook();

    #[cfg(feature="terminal")] {
        let highlight = highlight::highlight()?;
        for StyledTextRange{range, style} in highlight.style {
            fn print(text: &str, TextStyle{color, style}: TextStyle) {
                let code = match style {
                    FontStyle::Normal => 31,
                    FontStyle::Bold => 1,
                    //_ => 31
                };
                let bgra8{b,g,r,..} = color.into();
                print!("\x1b[{}m\x1b[38;2;{};{};{}m{}\x1b(B\x1b[m",code, r,g,b, text)
            }
            print(&highlight.text[range], style);
        }
    }
    #[cfg(feature="window")] {
        let highlight = highlight::highlight()?;
        use framework::{text::TextEdit, window::run};
        run(&mut TextEdit::new(&highlight.text, &highlight.style))?;
    }
    #[cfg(feature="iced")] {
        use self::iced::{Settings, Sandbox, Editor, window};
        Editor::run(Settings{window:window::Settings{overlay:true, ..Default::default()}, ..Default::default()})
    }
    log::trace!("editor: Ok");
}
