use ansi_term::Color;
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;

fn style_if_term(style: ansi_term::Style) -> ansi_term::Style {
    if *USE_COLOR {
        style
    } else {
        ansi_term::Style::new()
    }
}

pub(crate) fn create_download_progressbar(len: Option<u64>) -> ProgressBar {
    if let Some(len) = len {
        let p = indicatif::ProgressBar::new(len);
        p.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ETA {eta}")
                .progress_chars("=> "),
        );
        p
    } else {
        let p = indicatif::ProgressBar::new_spinner();
        p.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("{spinner:.green} {bytes} downloaded since {elapsed}"),
        );
        p
    }
}

pub fn create_progressbar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:20.red}] {pos}/{len}")
            .progress_chars("=> "),
    );
    pb
}

lazy_static! {
    pub static ref USE_COLOR: bool =
        atty::is(atty::Stream::Stdout) && atty::is(atty::Stream::Stderr);
    pub static ref ERROR_HEADER_STYLE: ansi_term::Style = style_if_term(Color::Red.bold());
    pub static ref WARNING_HEADER_STYLE: ansi_term::Style = style_if_term(Color::Yellow.bold());
    pub static ref SECONDARY: ansi_term::Style = style_if_term(Color::Cyan.bold());
    pub static ref HIGHLIGHT: ansi_term::Style = style_if_term(Color::White.bold());
}
