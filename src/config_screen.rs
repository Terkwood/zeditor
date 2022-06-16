use crate::screens::ZeditorScreens;
use cursive::{
    views::{Button, Dialog, DummyView, LinearLayout, TextView},
    Cursive,
};

pub fn render(siv: &mut Cursive, screens: ZeditorScreens) {
    siv.set_screen(screens.config);

    siv.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(TextView::new("Coming Soon"))
                .child(DummyView)
                .child(Button::new("Home", move |s| {
                    s.set_screen(screens.home);
                })),
        )
        .title("zeditor"),
    );
}
