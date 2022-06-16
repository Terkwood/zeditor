use cursive::{Cursive, ScreenId};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ZeditorScreens {
    Home(ScreenId),
    Config(ScreenId),
}

impl ZeditorScreens {
    /// run this once at the beginning of the program
    pub fn init(siv: &mut Cursive) -> (ZeditorScreens, ZeditorScreens) {
        (
            ZeditorScreens::Home(siv.active_screen()),
            ZeditorScreens::Config(siv.add_screen()),
        )
    }
}
