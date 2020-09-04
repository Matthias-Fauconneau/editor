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

struct Editor<'f, 't>{path: std::path::PathBuf, edit: Edit<'f,'t>}
impl Editor<'_, '_> {
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
impl Widget for Editor<'_, '_> {
	fn size(&mut self, size: size) -> size { self.edit.size(size) }
	#[throws] fn paint(&mut self, target: &mut Target) { self.edit.paint(target)? }
	#[throws] fn event(&mut self, size: size, event_context: &EventContext, event: &Event) -> bool { if self.event(size, event_context, event) != Change::None { true } else { false } }
}

struct CodeEditor<'f, 't>{editor: Editor<'f, 't>, diagnostics: Vec<rust::Diagnostic>, message: Option<String>, args: Vec<String>}
impl CodeEditor<'_, '_> {
	#[throws] fn update(&mut self) {
		let Self{editor: Editor{path, edit: Edit{view: View{data, ..}, ..}, ..}, diagnostics, ..} = self;
		*data = Cow::Owned(self::buffer(path)?);
		*diagnostics = rust::diagnostics(path)?;
		self.message = diagnostics.first().map(|rust::Diagnostic{message, ..}| message.clone());
	}
}

impl Widget for CodeEditor<'_, '_> {
	fn size(&mut self, size: size) -> size { self.editor.size(size) }
	#[throws] fn paint(&mut self, target: &mut Target) {
		let Self{editor: Editor{edit: Edit{view, selection, ..}, ..}, diagnostics, message, ..} = self;
		let scale = view.paint_fit(target);
		let text = AsRef::<str>::as_ref(&view.data);
		for rust::Diagnostic{range, ..} in diagnostics.iter() { view.paint_span(target, scale, from(text, *range), ui::color::bgr{b: false, g: false, r: true}); }
		view.paint_span(target, scale, *selection, ui::color::bgr{b: true, g: false, r: false});
		if let Some(message) = message {
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
				self.update()?;
				true
			}
			None => {
				let Self{editor: Editor{path, edit: Edit{view: View{data, ..}, selection, ..}}, diagnostics, args, ..} = self;
				let text = AsRef::<str>::as_ref(&data);
				let EventContext{modifiers_state: ModifiersState{alt,..}, ..} = event_context;
				match event {
					Event::Key{key:'→'} if *alt => {
						if let Some(rust::NavigationTarget{range, ..}) = dbg!(rust::definition(path, index(text, text::index(text, selection.end))))? { *selection = from(text, range); }
						true
					},
					Event::Key{key:'⎙'} => {
						if let Some(rust::Diagnostic{range, ..}) = diagnostics.first() { *selection = from(text, *range); }
						else if let Some(cargo::Diagnostic{message, spans, ..}, ..) = cargo::build(args)? {
							let cargo::Span{file_name, line_start, column_start, line_end, column_end, ..} = spans.into_iter().next().unwrap();
							*path = file_name.into();
							self.update()?;
							self.message = Some(message);
							self.editor.edit.selection = Span{start:LineColumn{line:line_start-1, column:column_start-1}, end:LineColumn{line:line_end-1, column:column_end-1}};
						} else {
							self.message = Option::None;
							std::process::Command::new("cargo").arg("run").spawn()?; // todo: stdout → message
						}
						true
					},
					_ => false
				}
			}
		}
	}
}

#[throws] fn main() {
	#[cfg(feature="trace")] trace::sigint();
	let mut args = std::env::args().skip(1);
	let path : Option<std::path::PathBuf> = args.next().map(|a| a.into());
	if let Some(path) = path.as_ref().filter(|p| p.is_dir()) { std::env::set_current_dir(path)?; }
	if let Some(path) = path.filter(|p| p.is_file()) {
		let text = std::fs::read(&path)?;
		run(Editor{path, edit: Edit::new(&default_font, Cow::Borrowed(Borrowed{text: &std::str::from_utf8(&text)?, style: &default_style}))})?
	} else {
		let path = Path::new("src/main.rs");
		run(CodeEditor{editor: Editor{path: path.to_path_buf(), edit: Edit::new(&default_font, Cow::Owned(buffer(path)?))}, diagnostics: rust::diagnostics(path)?, message: None,
															args: args.collect()})?
	}
}
