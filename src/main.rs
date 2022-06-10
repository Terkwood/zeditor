use cursive::reexports::crossbeam_channel::{unbounded, Sender};
use cursive::traits::*;
use cursive::views::{
    Button, Dialog, DummyView, LastSizeView, LinearLayout, ListChild, ListView, NamedView, Panel,
    TextView,
};
use cursive::{Cursive, CursiveRunnable, CursiveRunner};
use zeditor::search::{Hit, SearchFiles};

const SEARCH_RESULTS_WIDGET: &str = "search results";
// internally tracked count of items
const SEARCH_COUNT_WIDGET: &str = "search count";
// computed display size
const SEARCH_RESULTS_SIZE_WIDGET: &str = "search results size";
const SEARCH_RESULTS_SIZE_REPORT_WIDGET: &str = "search results size report";

const FILENAME_LABEL_LENGTH: usize = 15;

#[tokio::main]
async fn main() {
    let (search_files_s, search_files_r) = unbounded::<zeditor::search::SearchFiles>();
    let (files_searched_s, files_searched_r) = unbounded::<Vec<zeditor::search::Hit>>();

    tokio::spawn(async move { zeditor::search::run(files_searched_s, search_files_r).await });

    let (replace_hits_s, replace_hits_r) = unbounded::<zeditor::replace::ReplaceHits>();
    let (hits_replaced_s, hits_replaced_r) = unbounded::<zeditor::replace::HitsReplaced>();

    tokio::spawn(async move { zeditor::replace::run(hits_replaced_s, replace_hits_r).await });

    let mut siv = cursive::default().into_runner();

    const NO_SEARCH: Vec<Hit> = vec![];
    siv.set_user_data(NO_SEARCH);

    let search_count = TextView::new("Avail: 0").with_name(SEARCH_COUNT_WIDGET);

    let search_results = ListView::new().with_name(SEARCH_RESULTS_WIDGET);

    let search_results_size =
        LastSizeView::new(search_results).with_name(SEARCH_RESULTS_SIZE_WIDGET);

    let search_results_size_report =
        TextView::new("Max:   0").with_name(SEARCH_RESULTS_SIZE_REPORT_WIDGET);

    let perm_buttons = {
        let msg = search_files_s.clone();
        Panel::new(
            LinearLayout::vertical()
                .child(search_count)
                .child(search_results_size_report)
                .child(DummyView)
                .child(Button::new("Replace All", |s| bogus(s)))
                .child(Button::new("Search", move |_| {
                    msg.send(SearchFiles).unwrap()
                }))
                .child(DummyView)
                .child(Button::new("Quit", Cursive::quit)),
        )
    };

    siv.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(search_results_size)
                .child(DummyView)
                .child(perm_buttons),
        )
        .title("zeditor"),
    );

    refresh_search_list(&mut siv, &replace_hits_s);

    // manipulate the cursive event loop so that we can receive messages
    siv.refresh();
    while siv.is_running() {
        // update hacky counts & display size widgets
        update_hacky_widgets(&mut siv);

        siv.step();

        for files_searched in files_searched_r.try_iter() {
            update_search_list(&mut siv, files_searched, &replace_hits_s);
            // force refresh of UI
            siv.cb_sink().send(Box::new(Cursive::noop)).expect("send");
        }

        for _ in hits_replaced_r.try_iter() {
            // just clear the entire list and re-search
            update_search_list(&mut siv, vec![], &replace_hits_s);

            search_files_s.send(SearchFiles).expect("send");
        }
    }
}

fn refresh_search_list(siv: &mut Cursive, replace_hits_s: &Sender<zeditor::replace::ReplaceHits>) {
    if let Some(mut search_widget) = siv.find_name::<ListView>(SEARCH_RESULTS_WIDGET) {
        let _ = siv.with_user_data(|search_hits: &mut Vec<Hit>| {
            search_widget.clear();
            for (hit_pos, hit) in search_hits.clone().iter().enumerate() {
                let replace_hits_chan = replace_hits_s.clone();
                let replace_hits_chan2 = replace_hits_s.clone();
                let hitc = hit.clone();
                let linear = LinearLayout::horizontal()
                    .child(TextView::new(hit.preview.clone()))
                    .child(DummyView)
                    .child(Button::new("OK", move |_| {
                        replace_hits_chan
                            .send(zeditor::replace::ReplaceHits(vec![hitc.clone()]))
                            .expect("send")
                    }))
                    .child(DummyView)
                    .child(Button::new("Skip", move |s| {
                        skip_candidate(s, hit_pos, &replace_hits_chan2)
                    }));

                let label: String = hit
                    .path
                    .file_name()
                    .and_then(|o| o.to_str())
                    .unwrap_or("")
                    .trim()
                    .to_string()
                    .chars()
                    .into_iter()
                    .take(FILENAME_LABEL_LENGTH)
                    .collect();

                search_widget.add_child(&label, linear);
            }
        });
    }
}

fn update_search_list(
    siv: &mut Cursive,
    results: Vec<Hit>,
    replace_hits_s: &Sender<zeditor::replace::ReplaceHits>,
) {
    siv.with_user_data(|search_hits: &mut Vec<Hit>| {
        search_hits.clear();
        for f in results {
            search_hits.push(f);
        }
    });

    refresh_search_list(siv, replace_hits_s);
}

fn skip_candidate(
    siv: &mut Cursive,
    user_data_pos: usize,
    replace_hits_s: &Sender<zeditor::replace::ReplaceHits>,
) {
    siv.with_user_data(|hits: &mut Vec<Hit>| {
        hits.remove(user_data_pos);
    });
    refresh_search_list(siv, replace_hits_s);
}

fn bogus(_siv: &mut Cursive) {}

fn update_hacky_widgets(siv: &mut CursiveRunner<CursiveRunnable>) {
    let total_search_lines = count_visible_search_lines(siv);

    // update hacky count widget
    if let Some(mut search_count) = siv.find_name::<TextView>(SEARCH_COUNT_WIDGET) {
        search_count.set_content(format!("Avail: {}", total_search_lines));
    }

    // update hacky display size report widget
    if let Some(mut search_results_size_report) =
        siv.find_name::<TextView>(SEARCH_RESULTS_SIZE_REPORT_WIDGET)
    {
        if let Some(height) = find_search_results_height(siv) {
            search_results_size_report.set_content(format!("Max:   {}", height));
        } else {
            search_results_size_report.set_content("Error");
        }
    }

    // without this you'll lag behind by a step
    siv.refresh();
}

fn find_search_results_height(siv: &mut Cursive) -> Option<usize> {
    if let Some(search_results_size) =
        siv.find_name::<LastSizeView<NamedView<ListView>>>(SEARCH_RESULTS_SIZE_WIDGET)
    {
        Some(search_results_size.size.y)
    } else {
        None
    }
}

fn count_visible_search_lines(siv: &mut Cursive) -> usize {
    if let Some(search_widget) = siv.find_name::<ListView>(SEARCH_RESULTS_WIDGET) {
        let mut count = 0;
        for c in search_widget.children() {
            count += match c {
                ListChild::Delimiter => 1,
                ListChild::Row(_, v) => {
                    let ll: &LinearLayout = v.as_any().downcast_ref::<LinearLayout>().unwrap();
                    if let Some(text_child) = ll.get_child(0) {
                        let t: &TextView = text_child.as_any().downcast_ref::<TextView>().unwrap();

                        //  not really sure this will always work
                        // since it's using something about spans and styles
                        // https://docs.rs/cursive/latest/cursive/utils/span/struct.SpannedString.html
                        t.get_content().source().lines().into_iter().count()
                    } else {
                        0
                    }
                }
            };
        }

        count
    } else {
        0
    }
}
