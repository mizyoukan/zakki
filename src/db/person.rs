use errors::*;
use super::hash_password;

use postgres::GenericConnection;

#[derive(Debug, PartialEq)]
pub struct Person {
    pub id: i32,
    pub name: String,
}

impl Person {
    pub fn find(conn: &GenericConnection, name: &str, password: &str) -> Result<Person> {
        let hash = hash_password(password);
        let rows = conn.query(
            "SELECT id FROM person WHERE name = $1 AND password = $2",
            &[&name, &hash],
        )?;
        rows.iter()
            .next()
            .and_then(|row| {
                Some(Person {
                    id: row.get(0),
                    name: name.to_string(),
                })
            })
            .chain_err(|| "person does not exist")
    }

    pub fn get(conn: &GenericConnection, id: i32) -> Result<Person> {
        let rows = conn.query("SELECT name FROM person WHERE id = $1", &[&id])?;
        rows.iter()
            .next()
            .and_then(|row| {
                Some(Person {
                    id: id,
                    name: row.get(0),
                })
            })
            .chain_err(|| "person does not exist")
    }
}

#[cfg(test)]
mod tests {
    use db::testutil;
    use super::*;

    #[test]
    fn find() {
        testutil::with_db(|conn| {
            let person = Person::find(conn, "system", "manager").unwrap();
            assert_eq!(
                person,
                Person {
                    id: 1,
                    name: "system".into(),
                }
            );
        });
    }

    #[test]
    fn get() {
        testutil::with_db(|conn| {
            let person = Person::get(conn, 1).unwrap();
            assert_eq!(
                person,
                Person {
                    id: 1,
                    name: "system".into(),
                }
            );
        })
    }
}
