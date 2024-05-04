# Ruxt Macros
This is a collection of macros for the [Ruxt](https://ruxt.rs) web framework.

## Installation
Add the following to your `Cargo.toml`:
```toml
[dependencies]
ruxt-macros = "0.1.3"
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

## License
This project is licensed under the MIT license.
