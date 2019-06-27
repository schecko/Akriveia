extern crate yew;
extern crate common;

mod components;

use components::root::RootComponent;

fn main() {
    yew::start_app::<RootComponent>();
}

