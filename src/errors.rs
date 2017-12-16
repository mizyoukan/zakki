error_chain! {
    foreign_links {
        Postgres(::postgres::Error);
        R2d2(::r2d2::Error);
    }
}
