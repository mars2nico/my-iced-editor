mod my;

use iced::{Application, Font, Settings};
use my::*;

fn main() -> iced::Result {
    Editor::run(Settings {
        default_font: Font::MONOSPACE,
        #[rustfmt::skip]
        fonts: std::vec::Vec::from([
            include_bytes!("../fonts/editor-icons.ttf")
            .as_slice() // なぜ &[u8, N] から直接 Cow<'_, [u8]> に into できず、as_slice が必要なのか？
            .into(),
        ]),
        ..Settings::default()
    })
}
