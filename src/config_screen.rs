use cursive::traits::*;
use cursive::{
    views::{Button, Dialog, DummyView, LinearLayout, TextView},
    Cursive,
};

pub fn render(siv: &mut Cursive, home_screen_id: usize) {
    let hid = home_screen_id.clone();

    siv.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(TextView::new("hello"))
                .child(DummyView)
                .child(Button::new("Home", move |s| {
                    s.set_screen(hid);
                })),
        )
        .title("configure search terms"),
    );
}
