use {fehler::throws, error::Error, ui::text::{unicode_segmentation::find,Attribute,Style}};

#[cfg(feature="rust")] #[throws] fn buffer(path: &std::path::Path) -> ui::edit::Owned {
	let text = std::str::from_utf8(&std::fs::read(path)?)?.to_owned();
	use {rust::{HighlightedRange, HighlightTag, HighlightModifier}, ui::text::FontStyle, ui::color::bgr};
	pub fn style<'t>(text: &'t str, highlight: impl Iterator<Item=HighlightedRange>+'t) -> impl Iterator<Item=Attribute<Style>> + 't {
		highlight.map(move |HighlightedRange{range, highlight, ..}| {
			Attribute{
				range: find(text, range.start as usize)..find(text, range.end as usize),
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
	let style = style(&text, rust::highlight(path)?.into_iter()).collect::<Vec::<_>>();
	ui::edit::Owned{text, style}
}

#[cfg(not(feature="rust"))] #[throws] fn buffer(path: &std::path::Path) -> ui::edit::Owned {
	ui::edit::Owned{text: std::str::from_utf8(&std::fs::read(path)?)?.to_owned(), style: ui::text::default_style.to_vec()}
}
#[throws] fn main() {
	let path = std::path::Path::new("src/main.rs");
	let buffer = buffer(path)?;
	#[cfg(not(feature="app"))] println!("{:?}", buffer);
	#[cfg(feature="app")] ui::app::run(ui::edit::Edit::new(&ui::text::default_font, ui::edit::Cow::Owned(buffer), Some(Box::new(move |data| {
		std::fs::write(path, data.text.as_bytes()).unwrap();
		data.style = self::buffer(path).unwrap().style;
	}))))?
}
