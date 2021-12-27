# logger

## Installation

#### Dependencies

- [Rust with Cargo](http://rust-lang.org) `1.57.0` or later.

#### Importing

**Cargo.toml**

```toml
[dependencies]
logger = { version = "0.1.0", git = "https://github.com/kumanote/logger-rs", branch = "main", features = ["airbrake"] }
```

## Examples

Here's a basic example:

```rust
use logger::default::DefaultLoggerBuilder;
use logger::prelude::*;
use logger::setup_panic_logger;
use logger::Level;

use actix_web::{web, App, HttpServer};

fn double_number(number_str: &str) -> i32 {
    number_str
        .parse::<i32>()
        .map(|n| 2 * n)
        .expect("number_str must be valid number string")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut builder = DefaultLoggerBuilder::new();
    builder.is_async(true);
    builder.level(Level::Debug);
    builder.airbrake_host("https://api.airbrake.io".to_owned());
    builder.airbrake_project_id("<YOUR PROJECT ID>".to_owned());
    builder.airbrake_project_key("<YOUR API KEY>".to_owned());
    builder.airbrake_environment("production".to_owned());
    let _logger = builder.build();
    setup_panic_logger();

    HttpServer::new(move || App::new().configure(routes))
        .bind("0.0.0.0:8000")?
        .run()
        .await
}

pub fn routes(app: &mut web::ServiceConfig) {
    app.service(web::resource("/").route(web::get().to(index)));
}

pub async fn index(_req: web::HttpRequest) -> &'static str {
    debug!("this is a simple debug log");
    trace!("this is {}", "test");
    let value1 = 5;
    info!(key1 = value1);
    let _number = double_number("NOT A NUMBER");
    "OK"
}
```

**note**

This example uses `actix-web = "4.0.0-beta.19`.  
After you run the server, make a request to the local server by `curl localhost:8000`  
Then you will get log outputs in your console and crash information inside your airbrake project.

```bash
2022-01-14T11:04:35.133219Z [actix-rt|system:0|arbiter:0] 46964969 DEBUG app/src/main.rs:38 this is a simple debug log
2022-01-14T11:04:35.133272Z [actix-rt|system:0|arbiter:0] 46964969 INFO app/src/main.rs:41 {"key1":5}
2022-01-14T11:04:35.133411Z [actix-rt|system:0|arbiter:0] 46964969 CRASH logger-rs/src/crash_handler.rs:13 panicked at 'number_str must be valid number string: ParseIntError { kind: InvalidDigit }', app/src/main.rs:12:10
   0: logger::crash_handler::handle_panic
             at logger-rs/src/crash_handler.rs:13:5
   1: logger::crash_handler::setup_panic_logger::{{closure}}
             at logger-rs/src/crash_handler.rs:7:9
   2: std::panicking::rust_panic_with_hook
             at /rustc/f1edd0429582dd29cccacaf50fd134b05593bd9c/library/std/src/panicking.rs:628:17
   3: std::panicking::begin_panic_handler::{{closure}}
             at /rustc/f1edd0429582dd29cccacaf50fd134b05593bd9c/library/std/src/panicking.rs:521:13
   4: std::sys_common::backtrace::__rust_end_short_backtrace
             at /rustc/f1edd0429582dd29cccacaf50fd134b05593bd9c/library/std/src/sys_common/backtrace.rs:139:18
   5: rust_begin_unwind
             at /rustc/f1edd0429582dd29cccacaf50fd134b05593bd9c/library/std/src/panicking.rs:517:5
   6: core::panicking::panic_fmt
             at /rustc/f1edd0429582dd29cccacaf50fd134b05593bd9c/library/core/src/panicking.rs:100:14
   7: core::result::unwrap_failed
             at /rustc/f1edd0429582dd29cccacaf50fd134b05593bd9c/library/core/src/result.rs:1616:5
   8: core::result::Result<T,E>::expect
             at /rustc/f1edd0429582dd29cccacaf50fd134b05593bd9c/library/core/src/result.rs:1258:23
   9: app::double_number
             at app/src/main.rs:9:5
  10: app::index::{{closure}}
             at app/src/main.rs:42:19
...
```
