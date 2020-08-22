use {fehler::throws, error::Error, ui::text::{Attribute,Style}};
type Buffer = ui::edit::Buffer<std::sync::Arc<String>,Vec<Attribute<Style>>>;

#[cfg(feature="highlight")] #[throws] fn buffer() -> Buffer {
	use {highlight::{HighlightedRange, HighlightTag, HighlightModifier, TextHighlight}, ui::text::FontStyle, color::bgr};
	pub fn style(highlight: impl Iterator<Item=HighlightedRange>) -> impl Iterator<Item=Attribute<Style>> {
		highlight.map(|HighlightedRange{range, highlight, ..}| {
			Attribute{
				range: range.start().into()..range.end().into(),
				attribute: Style{
					color: {use HighlightTag::*; match highlight.tag {
						Module => bgr{b: 0., r: 1., g: 1./3.},
						Keyword if !highlight.modifiers.iter().any(|it| it == HighlightModifier::ControlFlow) => bgr{b: 2./3.,r: 2./3.,g: 2./3.},
						Function|Macro => bgr{b: 2./3., r: 1., g: 2./3.},
						Struct|TypeAlias|BuiltinType|TypeParam|Enum => bgr{b: 2./3., r: 0., g: 2./3.},
						Field => bgr{b: 0., r: 0.,g: 2./3.},
						Trait => bgr{b: 1., r: 1./2., g: 1.},
						StringLiteral|NumericLiteral|EnumVariant => bgr{b: 0., r: 1., g: 1./3.},
						Lifetime|Attribute => bgr{b: 1., r: 1./3., g: 1./3.},
						Comment => bgr{b: 1./2., r: 1./2., g: 1./2.},
						_ => bgr{b: 1., r: 1., g: 1.},
					}},
					style:
						if highlight.modifiers.iter().any(|it| it == HighlightModifier::ControlFlow) { FontStyle::Bold }
						else {
							{use HighlightTag::*; match highlight.tag {
									Keyword => FontStyle::Bold, // fixme: Italic
									_ => FontStyle::Normal
							}}
						}
				}
			}
		})
	}
	let TextHighlight{text, highlight} = highlight::highlight()?;
	Buffer{text, style: &style(highlight.into_iter()).collect::<Vec::<_>>()}
}

#[cfg(not(feature="highlight"))] #[throws] fn buffer() -> Buffer { Buffer{text: std::sync::Arc::new(std::str::from_utf8(&std::fs::read("src/main.rs")?)?.to_owned()), style: ui::text::default_style.to_vec()} }

#[throws] fn main() {
	let Buffer{text, style} = buffer()?;
	ui::app::run(ui::edit::Edit::new(&ui::text::default_font, ui::edit::Cow::Borrowed(ui::edit::Buffer{text: &text, style: &style})))?
}
