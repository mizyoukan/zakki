pub mod admin;

use db::Connection;
use db::article::Article;
use errors::*;

use ammonia::clean;
use maud::{html, Markup, Render, DOCTYPE};
use pulldown_cmark::{html, Parser};
use rocket::request::Request;
use rocket::response::NamedFile;
use std::path::{Path, PathBuf};

pub const APP_NAME: &'static str = "雑記";

struct Header<'a>(&'a str);

impl<'a> Render for Header<'a> {
    fn render(&self) -> Markup {
        let title = if self.0.is_empty() {
            APP_NAME.into()
        } else {
            format!("{} - {}", self.0, APP_NAME)
        };
        html! {
            (DOCTYPE)
            meta charset="utf-8";
            meta content="width=device-width" name="viewport";
            title (title)
            link href="/css/style.css" rel="stylesheet";
            header {
                h1 {
                    a href="/" (APP_NAME)
                }
            }
        }
    }
}

pub struct Markdown<'a>(&'a str);

impl<'a> Render for Markdown<'a> {
    fn render_to(&self, w: &mut String) {
        let parser = Parser::new(self.0);
        let mut html = String::new();
        html::push_html(&mut html, parser);
        let safe_html = clean(&html);
        w.push_str(&safe_html);
    }
}

pub struct ArticleView<'a>(&'a Article);

impl<'a> Render for ArticleView<'a> {
    fn render(&self) -> Markup {
        let article = self.0;
        html! {
            article {
                h1 {
                    a href={ "/article/" (article.id) } (article.title)
                }
                footer {
                    span {
                        "Posted on "
                        time datetime={ (article.created_at.format("%F")) } {
                            (article.created_at.format("%Y年%-m月%-d日"))
                        }
                    }
                }
                section { p (Markdown(&article.body)) }
            }
        }
    }
}

fn abbreviate_body(article: &mut Article) {
    let body = {
        let v: Vec<&str> = article.body.splitn(2, "<!-- more -->").collect();
        if v.len() == 1 {
            return;
        }
        format!("{}\n[続きを読む](/article/{})", v[0], article.id)
    };
    article.body = body;
}

#[get("/<file..>", rank = 99)]
pub fn static_file(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

#[get("/")]
pub fn index_page(conn: Connection) -> Result<Markup> {
    let mut articles = Article::list(&*conn)?;
    for mut article in articles.iter_mut() {
        abbreviate_body(&mut article);
    }

    Ok(html! {
        (Header(""))
        main {
            @for article in articles {
                (ArticleView(&article))
            }
        }
    })
}

#[get("/article/<id>")]
pub fn article_page(conn: Connection, id: i32) -> Result<Markup> {
    let article = Article::get(&*conn, id)?;
    Ok(html!{
        (Header(&article.title))
        main {
            (ArticleView(&article))
        }
    })
}

#[error(404)]
pub fn not_found(_: &Request) -> Markup {
    html! {
        (Header("404"))
        main {
            section "ページが見つかりません。"
        }
    }
}

#[error(500)]
pub fn internal_error() -> Markup {
    html! {
        (Header("500"))
        main {
            section "システムエラーが発生しました。"
        }
    }
}
