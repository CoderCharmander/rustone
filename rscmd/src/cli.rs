use ansi_term::Color;
use lazy_static::lazy_static;

fn style_if_term(style: ansi_term::Style) -> ansi_term::Style {
    if *USE_COLOR {
        style
    } else {
        ansi_term::Style::new()
    }
}

lazy_static! {
    pub static ref USE_COLOR: bool =
        atty::is(atty::Stream::Stdout) && atty::is(atty::Stream::Stderr);
    pub static ref ERROR_HEADER_STYLE: ansi_term::Style = style_if_term(Color::Red.bold());
    pub static ref WARNING_HEADER_STYLE: ansi_term::Style = style_if_term(Color::Yellow.bold());
    pub static ref SECONDARY: ansi_term::Style = style_if_term(Color::Cyan.bold());
    pub static ref HIGHLIGHT: ansi_term::Style = style_if_term(Color::White.bold());
}
