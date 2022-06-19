use cursive::reexports::crossbeam_channel::unbounded;
use cursive::Cursive;
use std::sync::{Arc, Mutex};
use zeditor::db::Db;
use zeditor::msg::Msg;
use zeditor::replace::{HitsReplaced, ReplaceHits};
use zeditor::screens::ZeditorScreens;
use zeditor::search::{Hit, SearchCommand};
use zeditor::skip::SkipRepo;
use zeditor::STATE;
use zeditor::{config_screen, home_screen};

#[tokio::main]
async fn main() {
    let db = Arc::new(Mutex::new(Db::new().expect("open db conn")));
    let db2 = db.clone();
    let db3 = db.clone();

    let skip_repo = Arc::new(Mutex::new(SkipRepo::new(db.clone())));

    let (search_files_s, search_files_r) = unbounded::<SearchCommand>();
    let (files_searched_s, files_searched_r) = unbounded::<Vec<Hit>>();

    tokio::spawn(async move { zeditor::search::run(db, files_searched_s, search_files_r).await });

    let (replace_hits_s, replace_hits_r) = unbounded::<Msg<ReplaceHits>>();
    let (hits_replaced_s, hits_replaced_r) = unbounded::<HitsReplaced>();

    tokio::spawn(async move { zeditor::replace::run(db2, hits_replaced_s, replace_hits_r).await });

    let mut siv = cursive::default().into_runner();

    siv.load_toml(include_str!("theme.toml")).unwrap();

    const NO_SEARCH: STATE = STATE(vec![]);
    siv.set_user_data(NO_SEARCH);

    let zeditor_screens = ZeditorScreens {
        home: siv.active_screen(),
        config: siv.add_screen(),
    };

    config_screen::render(
        &mut siv,
        zeditor_screens,
        db3,
        replace_hits_s.clone(),
        search_files_s.clone(),
    );

    home_screen::render(
        &mut siv,
        replace_hits_s.clone(),
        search_files_s.clone(),
        skip_repo.clone(),
        zeditor_screens,
    );

    // make sure we start on the home screen
    siv.set_screen(zeditor_screens.home);

    // manipulate the cursive event loop so that we can receive messages
    siv.refresh();
    while siv.is_running() {
        // update hacky counts & display size widgets
        home_screen::update_report_widgets(&mut siv);

        siv.step();

        for files_searched in files_searched_r.try_iter() {
            home_screen::update_found_user_data(
                &mut siv,
                files_searched,
                &replace_hits_s,
                skip_repo.clone(),
            );
            // force refresh of UI
            siv.cb_sink().send(Box::new(Cursive::noop)).expect("send");
        }

        for _ in hits_replaced_r.try_iter() {
            // just clear the entire list and re-search
            home_screen::update_found_user_data(
                &mut siv,
                vec![],
                &replace_hits_s,
                skip_repo.clone(),
            );

            search_files_s
                .send(SearchCommand::SearchFiles)
                .expect("send");
        }
    }
}
