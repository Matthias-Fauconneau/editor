use {std::path::Path, fehler::throws, error::Error,
				ui::{text::{self, unicode_segmentation::{index, find},Attribute,Style,View,LineColumn}, widget::{size, Target, EventContext, ModifiersState, Event, Widget}, edit::{Edit,Change,Span}}};

#[throws] fn buffer(path: &Path) -> ui::edit::Owned {
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

#[throws] fn main() {
	let path = Path::new("src/main.rs");
	let buffer = buffer(path)?;
	struct Editor<'t>{path: &'t Path, edit: Edit<'t,'t>};
	impl Widget for Editor<'_> {
		fn size(&mut self, size: size) -> size { self.edit.size(size) }
    fn paint(&mut self, target: &mut Target) -> Result<(),Error> { self.edit.paint(target) }
    #[throws] fn event(&mut self, size: size, event_context: &EventContext, event: &Event) -> bool {
			use Change::*;
			match self.edit.event(size, event_context, event) {
				Cursor => true,
				Insert|Remove|Other => {
					let Self{path, edit: Edit{view: View{data, ..}, ..}} = self;
					let text = AsRef::<str>::as_ref(&data);
					std::fs::write(&path, text.as_bytes()).unwrap();
					data.get_mut().style = self::buffer(path).unwrap().style;
					println!("{:?}", rust::diagnostics(path));
					true
				}
				None => {
					let Self{path, edit:Edit{view: View{data, ..}, selection, ..}} = self;
					let text = AsRef::<str>::as_ref(&data);
					let EventContext{modifiers_state: ModifiersState{alt,..}, ..} = event_context;
					match event {
						Event::Key{key:'→'} if *alt => {
							let target = dbg!(rust::definition(path, index(text, text::index(text, selection.end))))?;
							if let Some(target) = target { *selection = Span::new(LineColumn::from_text_index(text, find(text, target.range.start as usize)).unwrap()); }
							true
						},
						_ => false
					}
				}
			}
		}
	}
	ui::app::run(Editor{path, edit: ui::edit::Edit::new(&ui::text::default_font, ui::edit::Cow::Owned(buffer))})?
}
