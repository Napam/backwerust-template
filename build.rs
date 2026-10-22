fn main() {
    // Release builds embed the templates; debug builds read them from disk.
    // The macro tells cargo to rerun when anything under templates/ changes.
    minijinja_embed::embed_templates!("templates");
}
