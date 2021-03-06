//! Renders a bit of code as HTML.

use ra_db::SourceDatabase;
use ra_syntax::{AstNode, TextUnit};

use crate::{FileId, RootDatabase};

use super::highlight;

pub(crate) fn highlight_as_html(db: &RootDatabase, file_id: FileId, rainbow: bool) -> String {
    let parse = db.parse(file_id);

    fn rainbowify(seed: u64) -> String {
        use rand::prelude::*;
        let mut rng = SmallRng::seed_from_u64(seed);
        format!(
            "hsl({h},{s}%,{l}%)",
            h = rng.gen_range::<u16, _, _>(0, 361),
            s = rng.gen_range::<u16, _, _>(42, 99),
            l = rng.gen_range::<u16, _, _>(40, 91),
        )
    }

    let ranges = highlight(db, file_id, None);
    let text = parse.tree().syntax().to_string();
    let mut prev_pos = TextUnit::from(0);
    let mut buf = String::new();
    buf.push_str(&STYLE);
    buf.push_str("<pre><code>");
    for range in &ranges {
        if range.range.start() > prev_pos {
            let curr = &text[prev_pos.to_usize()..range.range.start().to_usize()];
            let text = html_escape(curr);
            buf.push_str(&text);
        }
        let curr = &text[range.range.start().to_usize()..range.range.end().to_usize()];

        let class = range.highlight.to_string().replace('.', " ");
        let color = match (rainbow, range.binding_hash) {
            (true, Some(hash)) => {
                format!(" data-binding-hash=\"{}\" style=\"color: {};\"", hash, rainbowify(hash))
            }
            _ => "".into(),
        };
        buf.push_str(&format!("<span class=\"{}\"{}>{}</span>", class, color, html_escape(curr)));

        prev_pos = range.range.end();
    }
    // Add the remaining (non-highlighted) text
    let curr = &text[prev_pos.to_usize()..];
    let text = html_escape(curr);
    buf.push_str(&text);
    buf.push_str("</code></pre>");
    buf
}

//FIXME: like, real html escaping
fn html_escape(text: &str) -> String {
    text.replace("<", "&lt;").replace(">", "&gt;")
}

const STYLE: &str = "
<style>
body                { margin: 0; }
pre                 { color: #DCDCCC; background: #3F3F3F; font-size: 22px; padding: 0.4em; }

.lifetime           { color: #DFAF8F; font-style: italic; }
.comment            { color: #7F9F7F; }
.struct, .enum      { color: #7CB8BB; }
.enum_variant       { color: #BDE0F3; }
.string_literal     { color: #CC9393; }
.field              { color: #94BFF3; }
.function           { color: #93E0E3; }
.parameter          { color: #94BFF3; }
.text               { color: #DCDCCC; }
.type               { color: #7CB8BB; }
.builtin_type       { color: #8CD0D3; }
.type_param         { color: #DFAF8F; }
.attribute          { color: #94BFF3; }
.numeric_literal    { color: #BFEBBF; }
.macro              { color: #94BFF3; }
.module             { color: #AFD8AF; }
.variable           { color: #DCDCCC; }
.mutable            { text-decoration: underline; }

.keyword            { color: #F0DFAF; font-weight: bold; }
.keyword.unsafe     { color: #BC8383; font-weight: bold; }
.control            { font-style: italic; }
</style>
";
