use lazy_static::lazy_static;

macro_rules! color_name {
    (red) => {
        ansi_term::Color::Red
    };
    (green) => {
        ansi_term::Color::Green
    };
    (blue) => {
        ansi_term::Color::Blue
    };
    (yellow) => {
        ansi_term::Color::Yellow
    };
    ($i:ident) => {
        ansi_term::Color::$i
    };
}

macro_rules! style_name {
    ($s:ident, bold) => {
        $s.bold()
    };
    ($s:ident, italic) => {
        $s.italic()
    };
    ($s:ident, underline) => {
        $s.underline()
    };
}

macro_rules! text_style_single {
    ($s:ident, color $i:ident) => {
        $s.fg(color_name! { $i })
    };

    ($s:ident, style $i:ident) => {
        style_name!($s, $i)
    };
}

macro_rules! text_style {
    ($($i:ident $j:ident),*) => {{
        let style = ansi_term::Style::new();
        $(
            let style = text_style_single!(style, $i $j);
        )*
        style
    }};
}

macro_rules! style_if_color {
    ($i:expr, $j:expr) => {
        if $i {
            $j
        } else {
            ansi_term::Style::new()
        }
    };
}

lazy_static! {
    pub static ref USE_COLOR: bool =
        atty::is(atty::Stream::Stdout) && atty::is(atty::Stream::Stderr);
    pub static ref ERROR_HEADER_STYLE: ansi_term::Style =
        style_if_color! {*USE_COLOR, text_style! {color red, style bold}};
    pub static ref WARNING_HEADER_STYLE: ansi_term::Style = if *USE_COLOR {
        text_style! (color yellow, style bold)
    } else {
        text_style!()
    };
    pub static ref SECONDARY: ansi_term::Style =
        style_if_color!(*USE_COLOR, text_style!(color Cyan, style bold));
}
