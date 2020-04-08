use {framework::*};
#[throws]
fn main() {
    let text = std::str::from_utf8(&std::fs::read("editor/src/main.rs")?)?.to_string();
    use ra_ide::HighlightedRange;
    for HighlightedRange{range, highlight, ..} in {let (analysis, file_id) = ra_ide::Analysis::from_single_file(text); analysis.highlight(file_id)?} {
        println!("{:?} {:?}", range, highlight);
    }
    //window(&mut Text::new(text))?
}
