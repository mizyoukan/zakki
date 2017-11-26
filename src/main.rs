#![feature(plugin)]
#![feature(proc_macro)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate maud;

use maud::{html, Markup, DOCTYPE};

#[get("/")]
fn index_page() -> Markup {
    html! {
        (DOCTYPE)
        html lang="ja" {
            head {
                meta charset="utf-8";
                title "雑記"
            }
            body {
                p "Hello, world!"
            }
        }
    }
}

fn main() {
    rocket::ignite().mount("/", routes![index_page]).launch();
}
