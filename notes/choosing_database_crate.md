# Choosing a Database Crate (p.48)

## Options

- `tokio-postgres`
- `sqlx`
- `diesel`

## Considerations

- Compile-Time safety
- SQL-First vs a DSL for query building
- Async vs Sync interface

### Compile-Time Safety

It is easy to make mistakes while interacting with a database!

- Typo
- Rejected operations
- NULL fields

_When_ do we realise there is a mistake? In most programming languages while we
are writing the raw SQL statements it is postponed to the runtime. It is the
case for `tokio-postgres`.

`diesel` and `sqlx` speeds up the feedback cycle and let us see errors at
**compile-time**. `diesel` depends on code generation of representation of
database schema. `sqlx` uses procedural macros to connect to db at compile-time
and check!

### Query Interface

- `tokio-postgres` and `sqlx` expects you to use _SQL_!
- `diesel` provides a query builder as a _DSL_.

#### Trade-off?

- You have to learn DSL first to use it!
- SQL statements are portable!
- SQL is not very easy to split into smaller unit and re-use...

#### Async Support

> Threads are for working in parallel, async is for waiting in parallel

Async db driver won't reduce how long it takes to process a single query, but it
will make you app to leverage all CPU cores to perform other things. _(e.g.
another HTTP request)_ while waiting for the db to return results.

_But_ async is complex, do you need that trade-off? It depends. If your web
framework is async, using an async db driver will be better though.

`sqlx` and `tokio-postgres` provides async interface, `diesel` not.
