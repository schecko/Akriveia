#![deny(warnings)]
// yew uses a lot of macros...
#![recursion_limit="256"]

extern crate yew;
extern crate common;
#[macro_use]
extern crate stdweb;
extern crate nalgebra as na;

#[macro_use]
mod util;
mod components;

use components::root::RootComponent;

fn main() {
    yew::start_app::<RootComponent>();
}

