use super::{Markdown, APP_NAME};
use db::Connection;
use db::article::Article;
use db::person::Person;
use errors::*;

use maud::{html, Markup, Render, DOCTYPE};
use rocket::http::{Cookie, Cookies};
use rocket::outcome::IntoOutcome;
use rocket::Request;
use rocket::request::{FlashMessage, Form, FromRequest, Outcome};
use rocket::response::{Flash, Redirect};
use std::path::PathBuf;

#[derive(FromForm)]
pub struct LoginForm {
    username: String,
    password: String,
}

pub struct LoginUser(i32);

impl<'a, 'r> FromRequest<'a, 'r> for LoginUser {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<LoginUser, ()> {
        request
            .cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|id| LoginUser(id))
            .or_forward(())
    }
}

pub struct ArticleView<'a>(&'a Article);

impl<'a> Render for ArticleView<'a> {
    fn render(&self) -> Markup {
        let article = self.0;
        html! {
            article {
                h1 {
                    a href={ "/admin/article/" (article.id) } (article.title)
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

#[derive(FromForm)]
pub struct ArticleForm {
    title: String,
    body: String,
}

struct AdminHeader<'a>(&'a str);

impl<'a> Render for AdminHeader<'a> {
    fn render(&self) -> Markup {
        html! {
            (DOCTYPE)
            meta charset="utf-8";
            meta content="width=device-width" name="viewport";
            @if self.0.is_empty() {
                title { (APP_NAME) " (管理)" }
            } else {
                title { (self.0) " - " (APP_NAME) " (管理)" }
            }
            link href="/css/admin.css" rel="stylesheet";
            header {
                h1 {
                    a href="/admin/" { (APP_NAME) " (管理)" }
                }
            }
        }
    }
}

#[get("/admin/login")]
pub fn login_page(flash: Option<FlashMessage>) -> Markup {
    html! {
        (AdminHeader("ログイン"))
        main {
            form#login action="login" method="post" {
                label for="username" "ユーザー名:"
                input#username type="text" name="username" autofocus="autofocus";
                label for="password" "パスワード:"
                input#password type="password" name="password";
                button type="submit" "ログイン"
                @if let Some(ref msg) = flash {
                    p class={ (msg.name()) "-message" } (msg.msg())
                }
            }
        }
    }
}

#[post("/admin/login", data = "<form>")]
pub fn login(mut cookies: Cookies, form: Form<LoginForm>, conn: Connection) -> Flash<Redirect> {
    let login = form.get();
    match Person::find(&*conn, &login.username, &login.password) {
        Ok(user) => {
            cookies.add_private(Cookie::new("user_id", user.id.to_string()));
            Flash::success(
                Redirect::to("/admin"),
                "ログインに成功しました。",
            )
        }
        Err(_) => Flash::error(
            Redirect::to("/admin/login"),
            "無効なユーザー名またはパスワードが指定されました。",
        ),
    }
}

#[post("/admin/logout")]
pub fn logout(mut cookies: Cookies) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named("user_id"));
    Flash::success(
        Redirect::to("/admin/login"),
        "ログアウトしました。",
    )
}

#[get("/admin", rank = 2)]
pub fn login_redirect() -> Redirect {
    Redirect::to("/admin/login")
}

#[get("/admin/<_path..>", rank = 2)]
pub fn nested_login_redirect(_path: PathBuf) -> Redirect {
    Redirect::to("/admin/login")
}

#[get("/admin")]
pub fn home_page(
    _login_user: LoginUser,
    flash: Option<FlashMessage>,
    conn: Connection,
) -> Result<Markup> {
    let articles = Article::list(&*conn)?;

    Ok(html! {
        (AdminHeader(""))
        main {
            @if !articles.is_empty() {
                h1 "記事の一覧"
                table {
                    thead {
                        tr {
                            th.title "タイトル"
                            th.created-at "登録日時"
                            th.updated-at "更新日時"
                            th colspan="2";
                        }
                    }
                    tbody {
                        @for article in articles {
                            tr {
                                td.title { a href={ "/admin/article/" (article.id) } (article.title) }
                                td.created-at (article.created_at.format("%F %T"))
                                td.updated-at (article.updated_at.format("%F %T"))
                                td.update { a href={ "/admin/article/update/" (article.id) } "編集" }
                                td.delete { a href={ "/admin/article/delete/" (article.id) } "削除" }
                            }
                        }
                    }
                }
            }
            a href="/admin/article/create" "記事を作成する"
        }
        footer {
            form action="/admin/logout" method="post" {
                button type="submit" "ログアウト"
            }
            @if let Some(ref msg) = flash {
                p class={ (msg.name()) "-message" } (msg.msg())
            }
        }
    })
}

#[get("/admin/article/<id>")]
pub fn preview_article_page(
    _login_user: LoginUser,
    id: i32,
    flash: Option<FlashMessage>,
    conn: Connection,
) -> Result<Markup> {
    let article = Article::get(&*conn, id)?;
    Ok(html!{
        (AdminHeader("記事の表示"))
        main {
            h1 "記事の表示"
            (ArticleView(&article))
            a href={ "/admin/article/update/" (article.id) } "編集"
            a href={ "/admin/article/delete/" (article.id) } "削除"
            @if let Some(ref msg) = flash {
                p class={ (msg.name()) "-message" } (msg.msg())
            }
        }
        footer {
            a href="/admin" "トップに戻る"
        }
    })
}

#[get("/admin/article/create")]
pub fn create_article_page(_login_user: LoginUser, flash: Option<FlashMessage>) -> Markup {
    html! {
        (AdminHeader("記事の作成"))
        main {
            h1 "記事の作成"
            form#article action="/admin/article/create" method="post" {
                label for="title" "タイトル:"
                input#title type="text" name="title" autofocus="autofocus";
                label for="body" "本文:"
                textarea#body name="body" {}
                button type="submit" "作成"
            }
            @if let Some(ref msg) = flash {
                p class={ (msg.name()) "-message" } (msg.msg())
            }
        }
        footer {
            a href="/admin" "トップに戻る"
        }
    }
}

#[post("/admin/article/create", data = "<form>")]
pub fn create_article(
    login_user: LoginUser,
    form: Form<ArticleForm>,
    conn: Connection,
) -> Result<Flash<Redirect>> {
    let article = form.get();
    if article.title.is_empty() {
        Ok(Flash::warning(
            Redirect::to("/admin/article/create"),
            "タイトルを入力してください。",
        ))
    } else {
        let tx = conn.transaction()?;
        let person = Person::get(&tx, login_user.0)?;
        Article::create(&tx, &article.title, &article.body, &person)?;
        tx.commit()?;
        Ok(Flash::success(
            Redirect::to("/admin"),
            "記事が作成されました。",
        ))
    }
}

#[get("/admin/article/update/<id>")]
pub fn update_article_page(
    _login_user: LoginUser,
    id: i32,
    flash: Option<FlashMessage>,
    conn: Connection,
) -> Result<Markup> {
    let article = Article::get(&*conn, id)?;
    Ok(html! {
        (AdminHeader("記事の編集"))
        main {
            h1 "記事の編集"
            form#article action={ "/admin/article/update/" (id) } method="post" {
                input type="hidden" name="_method" value="put";
                label for="title" "タイトル:"
                input#title type="text" name="title" value=(article.title);
                label for="body" "本文:"
                textarea#body name="body" (article.body)
                button type="submit" "編集"
            }
            @if let Some(ref msg) = flash {
                p class={ (msg.name()) "-message" } (msg.msg())
            }
        }
        footer {
            a href="/admin" "トップに戻る"
        }
    })
}

#[put("/admin/article/update/<id>", data = "<form>")]
pub fn update_article(
    _login_user: LoginUser,
    id: i32,
    form: Form<ArticleForm>,
    conn: Connection,
) -> Result<Flash<Redirect>> {
    let article = form.get();
    if article.title.is_empty() {
        Ok(Flash::warning(
            Redirect::to(&format!("/admin/article/update/{}", id)),
            "タイトルを入力してください。",
        ))
    } else {
        let tx = conn.transaction()?;
        Article::update(&tx, id, &article.title, &article.body)?;
        tx.commit()?;
        Ok(Flash::success(
            Redirect::to(&format!("/admin/article/{}", id)),
            "記事が編集されました。",
        ))
    }
}

#[get("/admin/article/delete/<id>")]
pub fn delete_article_page(_login_user: LoginUser, id: i32, conn: Connection) -> Result<Markup> {
    let article = Article::get(&*conn, id)?;
    Ok(html! {
        (AdminHeader("記事の削除"))
        main {
            h1 "記事の削除"
            (ArticleView(&article))
            p "記事を削除します。よろしいですか？"
            form action={ "/admin/article/delete/" (id) } method="post" {
                input type="hidden" name="_method" value="delete";
                button type="submit" "削除"
            }
        }
        footer {
            a href="/admin" "トップに戻る"
        }
    })
}

#[delete("/admin/article/delete/<id>")]
pub fn delete_article(
    _login_user: LoginUser,
    id: i32,
    conn: Connection,
) -> Result<Flash<Redirect>> {
    let tx = conn.transaction()?;
    Article::delete(&tx, id)?;
    tx.commit()?;
    Ok(Flash::success(
        Redirect::to("/admin"),
        "記事が削除されました。",
    ))
}
