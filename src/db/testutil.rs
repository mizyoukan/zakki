use postgres::{Connection, GenericConnection, TlsMode};

pub fn with_db<F: Fn(&GenericConnection)>(f: F) {
    let conn = Connection::connect(
        "postgres://postgres:Password12!@localhost:5432/zakki",
        TlsMode::None,
    ).unwrap();
    let tx = conn.transaction().unwrap();
    f(&tx);
}
