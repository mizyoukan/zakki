pub mod article;
pub mod person;

#[cfg(test)]
pub mod testutil;

use errors::*;

use openssl::ssl::{SslMethod, SslConnectorBuilder, SSL_VERIFY_NONE};
use postgres;
use postgres::tls::openssl::OpenSsl;
use r2d2;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use rocket::{Outcome, Request, State};
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use sha2::{Digest, Sha256};

use std::ops::Deref;

pub type Pool = r2d2::Pool<PostgresConnectionManager>;

pub struct Connection(r2d2::PooledConnection<PostgresConnectionManager>);

pub fn init_pool(database_url: &str) -> Result<Pool> {
    let mut connector = SslConnectorBuilder::new(SslMethod::tls())?;
    connector.set_verify(SSL_VERIFY_NONE);
    let openssl = OpenSsl::from(connector.build());
    let manager = PostgresConnectionManager::new(database_url, TlsMode::Prefer(Box::new(openssl)))?;
    let pool = r2d2::Pool::new(manager)?;
    Ok(pool)
}

impl Deref for Connection {
    type Target = postgres::Connection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Connection {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Connection, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(Connection(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

pub fn hash_password(password: &str) -> String {
    format!("{:x}", Sha256::digest_str(password))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_password_test() {
        assert_eq!(hash_password("password"),
                   "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8");
    }
}
