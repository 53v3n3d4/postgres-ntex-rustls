# deadpool-postgres + ntex + rustls example

Based on [deadpool-postgres example](https://github.com/bikeshedder/deadpool/tree/master/examples).  

Adapted to Ntex and Rustls.  

This example combines deadpool-postgres with a ntex webservice to implement a simple API service that responds with JSON read from PostgreSQL.

Instructions to run example, see [https://github.com/bikeshedder/deadpool/blob/master/examples/postgres-actix-web/README.md#running-the-example](https://github.com/bikeshedder/deadpool/blob/master/examples/postgres-actix-web/README.md#running-the-example).  

## Postgres

- Create postgres `deadpool` db.
- Load data `fixture.sql`
- Create `.env` file

## Run example

```
cargo run
```
