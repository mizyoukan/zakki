use errors::*;
use super::person::Person;

use chrono::{DateTime, Local};
use postgres::GenericConnection;

#[derive(Debug, PartialEq)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub author: Person,
}

impl Article {
    pub fn create(
        conn: &GenericConnection,
        title: &str,
        body: &str,
        author: &Person,
    ) -> Result<Article> {
        let rows = conn.query(
            "INSERT INTO article (title, body, author, created_at, updated_at)
                               VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                               RETURNING id, created_at, updated_at",
            &[&title, &body, &author.id],
        )?;
        rows.iter()
            .next()
            .and_then(|row| {
                Some(Article {
                    id: row.get(0),
                    title: title.to_string(),
                    body: body.to_string(),
                    created_at: row.get(1),
                    updated_at: row.get(2),
                    author: Person {
                        id: author.id,
                        name: author.name.to_string(),
                    },
                })
            })
            .chain_err(|| "failed to get article creation result")
    }

    pub fn update(conn: &GenericConnection, id: i32, title: &str, body: &str) -> Result<()> {
        match conn.execute(
            "UPDATE article SET title = $1, body = $2, updated_at = CURRENT_TIMESTAMP WHERE id = $3",
            &[&title, &body, &id],
        )? {
            1 => Ok(()),
            _ => Err("no article updated".into()),
        }
    }

    pub fn delete(conn: &GenericConnection, id: i32) -> Result<()> {
        match conn.execute("DELETE FROM article WHERE id = $1", &[&id])? {
            1 => Ok(()),
            _ => Err("no article deleted".into()),
        }
    }

    pub fn get(conn: &GenericConnection, id: i32) -> Result<Article> {
        let rows = conn.query(
            "SELECT a.id, a.title, a.body, a.created_at, a.updated_at, p.id, p.name
                               FROM article a JOIN person p ON p.id = a.author
                               WHERE a.id = $1",
            &[&id],
        )?;
        rows.iter()
            .next()
            .and_then(|row| {
                Some(Article {
                    id: row.get(0),
                    title: row.get(1),
                    body: row.get(2),
                    created_at: row.get(3),
                    updated_at: row.get(4),
                    author: Person {
                        id: row.get(5),
                        name: row.get(6),
                    },
                })
            })
            .chain_err(|| "person does not exist")
    }

    pub fn list(conn: &GenericConnection) -> Result<Vec<Article>> {
        let rows = conn.query(
            "SELECT a.id, a.title, a.body, a.created_at, a.updated_at, p.id, p.name
                               FROM article a JOIN person p ON p.id = a.author
                               ORDER BY a.updated_at DESC",
            &[],
        )?;
        let articles = rows.iter()
            .map(|row| Article {
                id: row.get(0),
                title: row.get(1),
                body: row.get(2),
                created_at: row.get(3),
                updated_at: row.get(4),
                author: Person {
                    id: row.get(5),
                    name: row.get(6),
                },
            })
            .collect();
        Ok(articles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::person::Person;
    use db::testutil;

    #[test]
    fn create() {
        testutil::with_db(|conn| {
            let author = Person {
                id: 1,
                name: "system".to_owned(),
            };
            let article = Article::create(conn, "title", "body", &author).unwrap();
            assert!(article.id > 0, "article id is present");
        });
    }

    #[test]
    fn update() {
        testutil::with_db(|conn| {
            let article = Article::create(
                conn,
                "title",
                "body",
                &Person {
                    id: 1,
                    name: "system".to_owned(),
                },
            ).unwrap();
            Article::update(conn, article.id, "title2", "body2").unwrap();
            let result = conn.query(
                "SELECT title, body FROM article WHERE id = $1",
                &[&article.id],
            ).unwrap()
                .iter()
                .next()
                .map(|row| (row.get(0), row.get(1)));
            assert_eq!(result, Some(("title2".to_owned(), "body2".to_owned())));
        });
    }

    #[test]
    fn delete() {
        testutil::with_db(|conn| {
            conn.execute(
                "INSERT INTO article (id, title, body, author, created_at, updated_at)
                          VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                &[&1, &"title", &"body", &1],
            ).unwrap();
            Article::delete(conn, 1).unwrap();
        });
    }

    #[test]
    fn get() {
        testutil::with_db(|conn| {
            conn.execute(
                "INSERT INTO article (id, title, body, author, created_at, updated_at)
                          VALUES ($1, $2, $3, $4, '2000-01-02 03:04:05.006+09', '2017-12-01 12:34:56.789+09')",
                &[&1, &"title", &"body", &1],
            ).unwrap();
            let expected = Article {
                id: 1,
                title: "title".to_owned(),
                body: "body".to_owned(),
                created_at: "2000-1-2T03:04:05.006+09:00"
                    .parse::<DateTime<Local>>()
                    .unwrap(),
                updated_at: "2017-12-1T12:34:56.789+09:00"
                    .parse::<DateTime<Local>>()
                    .unwrap(),
                author: Person {
                    id: 1,
                    name: "system".to_owned(),
                },
            };
            let actual = Article::get(conn, 1).unwrap();
            assert_eq!(expected, actual);
        });
    }

    #[test]
    fn list() {
        testutil::with_db(|conn| {
            conn.execute("DELETE FROM article", &[]).unwrap();
            conn.execute(
                "INSERT INTO article (title, body, author, created_at, updated_at)
                          VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                &[&"title", &"body", &1],
            ).unwrap();
            let articles = Article::list(conn).unwrap();
            assert_eq!(articles.len(), 1);
        });
    }
}
