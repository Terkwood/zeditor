pub mod config_screen;
pub mod db;
mod env;
pub mod home_screen;
pub mod msg;
mod quit;
pub mod replace;
pub mod screens;
pub mod search;
pub mod skip;

pub struct STATE(pub Vec<crate::search::Hit>);
