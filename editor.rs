use {fehler::throws, std::path::{Path, PathBuf},
		ui::{Error, text::{self, unicode_segmentation::{index, find},Attribute,Style,bgr,FontStyle,View,Borrowed,LineColumn,Span,default_font},
		widget::{size, int2, xy, Target, EventContext, ModifiersState, Event, Widget},
		edit::{Owned,Cow,Scroll,Edit,Change}, run}};

#[throws] fn buffer(path: &Path) -> Owned {
	let text = String::from_utf8(std::fs::read(path)?)?;
	use rust::{HlRange, HlTag, SymbolKind, HlMod};
	fn style(_text: &str, HlRange{range, highlight, ..}:&HlRange) -> Attribute<Style> { Attribute{
		range: u32::from(range.start()) as usize .. u32::from(range.end()) as usize,
		attribute: Style{
			color: {use {HlTag::*, SymbolKind::*}; match highlight.tag {
				Symbol(Module) => bgr{b: 1./3., g: 2./3., r: 1./3.},
				Keyword => { if !highlight.mods.iter().any(|it| it == HlMod::ControlFlow) { bgr{b: 2./3., g: 2./3., r: 2./3.} } else { bgr{b: 0., g: 1., r: 1.} } },
				Symbol(Function|Macro) => bgr{b: 2./3., g: 2./3., r: 1.},
				Symbol(Struct|TypeAlias|TypeParam|Enum)|BuiltinType => bgr{b: 2./3., g: 2./3., r: 0.},
				Symbol(Field) => bgr{b: 0., g: 2./3., r: 0.,},
				Symbol(Trait) => bgr{b: 1., g: 1., r: 1./2.,},
				BoolLiteral|ByteLiteral|CharLiteral|StringLiteral|NumericLiteral|Symbol(Variant) => bgr{b: 0., g: 1./3., r: 2./3.},
				Symbol(LifetimeParam)|AttributeBracket|Symbol(Attribute)|Symbol(BuiltinAttr) => bgr{b: 1., g: 1./3., r: 1./3.,},
				Symbol(_)|FormatSpecifier|Operator(_)|UnresolvedReference|None => bgr{b: 1., g: 1., r: 1.,},
				Punctuation(_)|EscapeSequence => bgr{b: 1./2., g: 1., r: 1./2.},
				Comment => bgr{b: 1./2., g: 1./2., r: 1./2.,},
			}},
			style:
				if highlight.mods.iter().any(|it| it == HlMod::ControlFlow) { FontStyle::Bold }
				else {
					{use HlTag::*; match highlight.tag {
							Keyword => FontStyle::Bold, // fixme: Italic
							_ => FontStyle::Normal
					}}
				}
		}
	}}
	let time = std::time::Instant::now();
	let style = rust::highlight(rust::file_id(path)?.unwrap())?.into_iter().map(|range| style(&text, range)).collect();
	//eprintln!("highlight {:?}", (std::time::Instant::now()-time));
	Owned{text, style}
}

#[track_caller] fn from_index(text: &str, byte_index: rust::TextSize) -> LineColumn { LineColumn::from_text_index(text, find(text, byte_index)).unwrap() }
fn from(text: &str, range: rust::TextRange) -> Span { Span{start: from_index(text, range.start().into()), end: from_index(text, range.end().into())} }

#[derive(derive_more::Deref)] struct Editor<'f, 't>{
	path: std::path::PathBuf,
	#[deref] scroll: Scroll<'f,'t>
}
impl Editor<'_, '_> {
	fn event(&mut self, size: size, event_context: &mut EventContext, event: &Event) -> Change {
		let change = self.scroll.event(size, event_context, event);
		if let Change::Insert|Change::Remove|Change::Other = change {
				let Self{path, scroll: Scroll{edit: Edit{view, ..}, ..}} = self;
				std::fs::write(&path, view.text().as_bytes()).unwrap();
				rust::change(rust::file_id(path).unwrap().unwrap()).unwrap();
		}
		change
	}
}
impl Widget for Editor<'_, '_> {
	fn size(&mut self, size: size) -> size { self.scroll.size(size) }
	#[throws] fn paint(&mut self, target: &mut Target, size: size, offset: int2) { self.scroll.paint(target, size, offset)? }
	#[throws] fn event(&mut self, size: size, event_context: &mut EventContext, event: &Event) -> bool {
		if self.event(size, event_context, event) != Change::None { true } else { false }
	}
}

struct CodeEditor<'f, 't>{
	editor: Editor<'f, 't>,
	diagnostics: Box<[rust::Diagnostic]>,
	message: Option<String>,
	args: Vec<String>,
	/*selection/browse_*/history: Vec<(PathBuf, Span)>,
}
impl CodeEditor<'_, '_> {
	#[throws] fn update(&mut self) {
		let Self{editor: Editor{path, scroll: Scroll{edit: Edit{view, ..}, ..}}, diagnostics, message, ..} = self;
		view.size = None;
		let time = std::time::Instant::now();
		view.data = Cow::Owned(self::buffer(path)?);
		//eprintln!("highlight {:?}", (std::time::Instant::now()-time));
		*diagnostics = rust::diagnostics(rust::file_id(path)?.unwrap())?;
		*message = diagnostics.first().map(|rust::Diagnostic{message, ..}| message.clone());
		//eprintln!("done");
	}
	#[throws] fn view(&mut self, path: PathBuf) {
		self.editor.path = path;
		self.update()?
	}
}

impl Widget for CodeEditor<'_, '_> {
	fn size(&mut self, size: size) -> size { self.editor.size(size) }
	#[throws] fn paint(&mut self, target: &mut Target, size: size, offset: int2) {
		//target.fill(0.into());
		let Self{editor: Editor{scroll, ..}, diagnostics, message, ..} = self;
		let scale = scroll.paint_fit(target, size, offset);
		let Scroll{edit: Edit{view, selection, ..}, offset} = scroll;
		for rust::Diagnostic{range, ..} in diagnostics.iter() {
			view.paint_span(target, scale, offset.signed(), from(view.text(), *range), ui::color::bgr{b: false, g: false, r: true});
		}
		view.paint_span(target, scale, offset.signed(), *selection, ui::color::bgr{b: true, g: true, r: true});
		if let Some(text) = message {
			let mut view = View::new(Borrowed{text,style:&[]});
			let text_size = text::fit(size, view.size());
			Widget::paint(&mut view, target, xy{x: size.x, y: text_size.y}, xy{x: 0, y: (size.y-text_size.y) as i32})?;
		}
	}
	#[throws] fn event(&mut self, size: size, event_context: &mut EventContext, event: &Event) -> bool {
		match self.editor.event(size, event_context, event) {
			Change::Cursor|Change::Scroll => true,
			Change::Insert|Change::Remove|Change::Other => {
				rust::change(rust::file_id(&self.editor.path)?.unwrap())?;
				self.update()?;
				true
			}
			Change::None => {
				let EventContext{modifiers_state: ModifiersState{alt,..}, ..} = event_context;
				let Self{editor: Editor{path, scroll: Scroll{edit: Edit{view, selection, ..}, ..}}, diagnostics, ref args, history, ..} = self;
				let text = view.text();
				match event {
					Event::Key('←') if *alt => {
						if let Some((path, span)) = history.pop() {
							self.view(path)?;
							let scroll = &mut self.editor.scroll;
							scroll.edit.selection = span;
							scroll.keep_selection_in_view(size);
						}
						true
					},
					Event::Key('→') if *alt => {
						if let Some(target) = rust::definition(rust::FilePosition{file_id: rust::file_id(path)?.unwrap(), offset: index(text, text::index(text, selection.end)).try_into().unwrap()})? {
							history.push((path.clone(), *selection));
							let rust::NavigationTarget{path, range, ..} = target;
							self.view(path)?;
							let span = Span::new(from_index(self.editor.view.text(), range.start()));
							let scroll = &mut self.editor.scroll;
							scroll.edit.selection = span;
							scroll.keep_selection_in_view(size);
						}
						true
					},
					Event::Key('⎙') => {
						if let Some(rust::Diagnostic{range, ..}) = diagnostics.first() { *selection = from(text, *range); }
						else if let Err(cargo::Diagnostic{message, spans, ..}, ..) = cargo::build(args)? {
							let cargo::Span{file_name, line_start, column_start, line_end, column_end, ..} = spans.into_iter().next().unwrap();
							self.view(file_name.into())?;
							self.message = Some(message);
							let scroll = &mut self.editor.scroll;
							scroll.edit.selection = Span{start:LineColumn{line:line_start-1, column:column_start-1}, end:LineColumn{line:line_end-1, column:column_end-1}};
							scroll.keep_selection_in_view(size);
						} else {
							// TODO: restart (terminate any previous still running instance)
							self.message = None;
							std::process::Command::new("cargo").args(args).arg("run").spawn()?; // todo: stdout → message
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
	let time = std::time::Instant::now();
	#[cfg(feature="trace")] { trace::rstack_self().unwrap(); trace::signal_interrupt()?; }
	let mut args = std::env::args().skip(1);
	let path : std::path::PathBuf = args.next().map(|a| a.into()).unwrap_or(std::env::current_dir()?);
	if let Some(project) = path.canonicalize()?.ancestors().find(|p| p.join("Cargo.toml").is_file()) {
		std::env::set_current_dir(project)?;
		let path =
			if path.is_file() { path } else {
				["main.rs","src/main.rs"].iter().map(|path| project.join(path)).filter(|path| path.exists()).next()
				.unwrap_or_else(|| cargo::parse(&project.join("Cargo.toml")).unwrap().into())
			};
		let mut code = CodeEditor{editor: Editor{path, scroll: Scroll::new(Edit::new(default_font(), Cow::new("")))}, diagnostics: Box::new([]), message: None, args: args.collect(), history: vec![]};
		code.update()?;
		//eprintln!("update {:?}", (std::time::Instant::now()-time));
		run(&mut code)?
	} else {
		let text = std::fs::read(&path)?;
		run(&mut Editor{path, scroll: Scroll::new(Edit::new(default_font(), Cow::Owned(Owned{text: String::from_utf8(text)?, style: Vec::new()})))})?
	}
}
