use cursive::traits::*;
use cursive::views::{
    Button, Dialog, DummyView, EditView, LinearLayout, ListView, SelectView, TextArea,
};
use cursive::Cursive;

#[derive(Clone)]
struct ReplacementCandidate {
    search: String,
    preview_blurb: String,
}

fn main() {
    let mut siv = cursive::default();

    let select = SelectView::<String>::new()
        .on_submit(on_name_click)
        .with_name("select")
        .fixed_size((10, 5));

    siv.set_user_data(vec![ReplacementCandidate {
        search: "scala".to_string(),
        preview_blurb: "scala is a lang".to_string(),
    }]);

    let fake_stuff = ListView::new().with_name("fake_stuff");

    let buttons = LinearLayout::vertical()
        .child(Button::new("Add new", add_name))
        .child(Button::new("Delete", delete_name))
        .child(DummyView)
        .child(Button::new("Fake", |s| {
            update_fake_db(
                s,
                ReplacementCandidate {
                    search: "rust".to_string(),
                    preview_blurb: "but rust is better".to_string(),
                },
            )
        }))
        .child(DummyView)
        .child(Button::new("Quit", Cursive::quit));

    siv.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(select)
                .child(DummyView)
                .child(buttons)
                .child(DummyView)
                .child(fake_stuff),
        )
        .title("zeditor"),
    );

    refresh_fake_list(&mut siv);

    siv.run();
}

fn add_name(s: &mut Cursive) {
    fn ok(s: &mut Cursive, name: &str) {
        s.call_on_name("select", |view: &mut SelectView<String>| {
            view.add_item_str(name)
        });
        s.pop_layer();
    }

    s.add_layer(
        Dialog::around(
            EditView::new()
                .on_submit(ok)
                .with_name("name")
                .fixed_width(10),
        )
        .title("Enter a new name")
        .button("Ok", |s| {
            let name = s
                .call_on_name("name", |view: &mut EditView| view.get_content())
                .unwrap();
            ok(s, &name);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}

fn delete_name(s: &mut Cursive) {
    let mut select = s.find_name::<SelectView<String>>("select").unwrap();
    match select.selected_id() {
        None => s.add_layer(Dialog::info("No name to remove")),
        Some(focus) => {
            select.remove_item(focus);
        }
    }
}

fn refresh_fake_list(siv: &mut Cursive) {
    if let Some(mut fake_stuff) = siv.find_name::<ListView>("fake_stuff") {
        let _ = siv.with_user_data(|blurbs: &mut Vec<ReplacementCandidate>| {
            fake_stuff.clear();
            for b in blurbs {
                let linear = LinearLayout::horizontal()
                    .child(TextArea::new().content(b.preview_blurb.clone()))
                    .child(DummyView)
                    .child(Button::new("OK", bogus))
                    .child(DummyView)
                    .child(Button::new("Skip", bogus));

                fake_stuff.add_child(&b.search, linear)
            }
        });
    }
}

fn update_fake_db(siv: &mut Cursive, input: ReplacementCandidate) {
    siv.with_user_data(|blurbs: &mut Vec<ReplacementCandidate>| blurbs.push(input.clone()));
    refresh_fake_list(siv);
}

fn bogus(_siv: &mut Cursive) {}

fn on_name_click(s: &mut Cursive, name: &str) {
    s.add_layer(
        Dialog::text(format!("Name: {}\nAwesome: maybe", name))
            .title(format!("{}'s info", name))
            .button("Ok", |s| {
                s.pop_layer();
            }),
    );
}
