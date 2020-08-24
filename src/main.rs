use {std::path::Path, fehler::throws, error::Error,
				ui::{text::{self, unicode_segmentation::{index, find},Attribute,Style,View,LineColumn,Span,default_font, default_style},
				widget::{size, Target, EventContext, ModifiersState, Event, Widget},
				edit::{Borrowed,Cow,Edit,Change}, app::run}};

#[throws] fn buffer(path: &Path) -> ui::edit::Owned {
	let text = String::from_utf8(std::fs::read(path)?)?;
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

fn from_index(text: &str, byte_index: usize) -> LineColumn { LineColumn::from_text_index(text, find(text, byte_index)).unwrap() }
fn from(text: &str, range: rust::TextRange) -> Span { Span{start: from_index(text, range.start as usize), end: from_index(text, range.end as usize)} }

struct Editor<'t>{path: &'t Path, edit: Edit<'t,'t>}
impl Editor<'_> {
	fn event(&mut self, size: size, event_context: &EventContext, event: &Event) -> Change {
		let change = self.edit.event(size, event_context, event);
		use Change::*;
		if let Insert|Remove|Other = change {
				let Self{path, edit: Edit{view: View{data, ..}, ..}} = self;
				let text = AsRef::<str>::as_ref(&data);
				std::fs::write(&path, text.as_bytes()).unwrap();
		}
		change
	}
}
impl Widget for Editor<'_> {
	fn size(&mut self, size: size) -> size { self.edit.size(size) }
	#[throws] fn paint(&mut self, target: &mut Target) { self.edit.paint(target)? }
	#[throws] fn event(&mut self, size: size, event_context: &EventContext, event: &Event) -> bool { if self.event(size, event_context, event) != Change::None { true } else { false } }
}

struct CodeEditor<'t>{editor: Editor<'t>, diagnostics: Vec<rust::Diagnostic>}
impl Widget for CodeEditor<'_> {
	fn size(&mut self, size: size) -> size { self.editor.size(size) }
	#[throws] fn paint(&mut self, target: &mut Target) {
		let Self{editor: Editor{edit: Edit{view, selection, ..}, ..}, diagnostics, ..} = self;
		let scale = view.scale(target.size);
		view.paint(target, scale);
		let text = AsRef::<str>::as_ref(&view.data);
		for rust::Diagnostic{range, ..} in diagnostics.iter() { view.paint_span(target, scale, from(text, *range), ui::color::bgr{b: false, g: false, r: true}); }
		view.paint_span(target, scale, *selection, ui::color::bgr{b: true, g: false, r: false});
		if let Some(rust::Diagnostic{message, ..}) = diagnostics.first() {
			let mut view = View{font: &default_font, data: 	Borrowed{text: message, style: &default_style}};
			let size = text::fit(target.size, view.size());
			Widget::paint(&mut view, &mut target.rows_mut(target.size.y-size.y..target.size.y))?;
		}
	}
	#[throws] fn event(&mut self, size: size, event_context: &EventContext, event: &Event) -> bool {
		use Change::*;
		match self.editor.event(size, event_context, event) {
			Cursor => true,
			Insert|Remove|Other => {
				let Self{editor: Editor{path, edit: Edit{view: View{data, ..}, ..}, ..}, diagnostics} = self;
				data.get_mut().style = self::buffer(path).unwrap().style;
				*diagnostics = rust::diagnostics(path)?;
				true
			}
			None => {
				let Self{editor: Editor{path, edit: Edit{view: View{data, ..}, selection, ..}}, diagnostics} = self;
				let text = AsRef::<str>::as_ref(&data);
				let EventContext{modifiers_state: ModifiersState{alt,..}, ..} = event_context;
				match event {
					Event::Key{key:'→'} if *alt => {
						if let Some(rust::NavigationTarget{range, ..}) = dbg!(rust::definition(path, index(text, text::index(text, selection.end))))? { *selection = from(text, range); }
						true
					},
					Event::Key{key:'⎙'} => {
						if let Some(rust::Diagnostic{range, ..}) = diagnostics.first() { *selection = from(text, *range); true }
						else { std::process::Command::new("cargo").arg("run").spawn()?; false }
					},
					_ => false
				}
			}
		}
	}
}

#[throws] fn main() {
	let path : Option<std::path::PathBuf> = std::env::args().nth(1).map(|a| a.into());
	if let Some(path) = path.as_ref().filter(|p| p.is_dir()) { std::env::set_current_dir(path)?; }
	if let Some(path) = path.filter(|p| p.is_file()) {
		let text = std::fs::read(&path)?;
		run(Editor{path: &path, edit: Edit::new(&default_font, Cow::Borrowed(Borrowed{text: &std::str::from_utf8(&text)?, style: &default_style}))})?
	} else {
		let path = Path::new("src/main.rs");
		run(CodeEditor{editor: Editor{path, edit: Edit::new(&default_font, Cow::Owned(buffer(path)?))}, diagnostics: rust::diagnostics(path)?})?
	}
}
