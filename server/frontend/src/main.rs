extern crate yew;
extern crate common;

#[macro_use]
mod util;
mod components;

use components::root::RootComponent;

fn main() {
    yew::start_app::<RootComponent>();
}

