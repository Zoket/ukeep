use crate::pages::{AddItem, Home};
use dioxus::prelude::*;

#[derive(Routable, Clone)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/add")]
    AddItem {},
}
