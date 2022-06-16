use cursive::{
    views::{Dialog, DummyView, LinearLayout, TextView},
    Cursive,
};

pub fn render(siv: &mut Cursive) {
    siv.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(TextView::new("hello"))
                .child(DummyView),
        )
        .title("config"),
    );
}
