# Ruxt Macros
This is a collection of macros for the [Ruxt](https://ruxt.rs) web framework.

## Installation
Run the following command to add Ruxt to your project:
```bash
cargo add ruxt
```

Add the following to your `Cargo.toml`:
```toml
[dependencies]
ruxt-macros = "0.1.4"
```

## Usage
```rust
#[ruxt_macros::main]
async fn main() -> std::io::Result<()> {
   let test_data = "Hello, World!";
   HttpServer::new(move || App::new().app_data(test_data.to_string()))
   .bind(("0.0.0.0", 8080))?
   .run()
   .await
}
```

## Basic Routing
The Ruxt `main` macro will automatically generate routes for files in the `routes` directory.
The routes are generated based on the file name, so a file named `index.rs` will be available at the root of the server.

The macro determines which HTTP verb to use based on the function name. For example, a function named `get` will be a `GET` route.

So for example:

```rust
// routes/index.rs
use actix_web::{web, HttpResponse, Responder};

pub async fn get() -> impl Responder {
    HttpResponse::Ok().body("Hello, World!")
}
```

Will be available as a GET request at `http://localhost:8080/`.

The following verbs are available for routing:
- `get`
- `post`
- `put`
- `patch`
- `delete`

## Dynamic Paths
Dynamic routes can be created by naming a folder or file with two leading underscores. For example, a folder named `__user` will create a dynamic route at `/user/{id}`.

```rust
// routes/__user.rs
use actix_web::{web, HttpResponse, Responder};

pub async fn post(id: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("Hello, {}!", id))
}
```

Will be available as a POST request at `http://localhost:8080/user/{id}`.

## Current Limitations
- As of now it is not possible to have a route with the name `mod` or `index` because of the way the macro generates routes. I'm looking into a solution for this.

## License
This project is licensed under the MIT license.
