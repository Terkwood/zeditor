use cursive::{
    views::{Button, Dialog, DummyView, LinearLayout, TextView},
    Cursive,
};

pub fn render(siv: &mut Cursive, home_screen_id: usize, config_screen_id: usize) {
    siv.set_screen(config_screen_id);

    let hid = home_screen_id.clone();

    siv.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(TextView::new("Coming Soon"))
                .child(DummyView)
                .child(Button::new("Home", move |s| {
                    s.set_screen(hid);
                })),
        )
        .title("zeditor"),
    );
}
