#![feature(custom_derive)]
#![feature(plugin)]
#![feature(proc_macro)]
#![plugin(rocket_codegen)]
#![recursion_limit = "1024"]

extern crate ammonia;
extern crate chrono;
#[macro_use]
extern crate error_chain;
extern crate maud;
extern crate openssl;
extern crate postgres;
extern crate pulldown_cmark;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate rocket;
extern crate sha2;

#[cfg(test)]
extern crate toml;

mod db;
mod errors;
mod views;

use rocket::fairing::AdHoc;
use rocket::Rocket;

fn rocket() -> Rocket {
    rocket::ignite()
        .attach(AdHoc::on_attach(|rocket| {
            let pool = {
                let database_url = rocket.config().get_str("database_url").unwrap();
                db::init_pool(database_url).unwrap()
            };
            Ok(rocket.manage(pool))
        }))
        .mount("/", routes![
            views::static_file,
            views::index_page,
            views::article_page,
            views::admin::login_page,
            views::admin::login,
            views::admin::logout,
            views::admin::login_redirect,
            views::admin::nested_login_redirect,
            views::admin::home_page,
            views::admin::preview_article_page,
            views::admin::create_article_page,
            views::admin::create_article,
            views::admin::update_article_page,
            views::admin::update_article,
            views::admin::delete_article_page,
            views::admin::delete_article,
        ])
        .catch(errors![views::not_found, views::internal_error])
}

fn main() {
    rocket().launch();
}
