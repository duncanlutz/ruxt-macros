use proc_macro::TokenStream;
use quote::quote;
use std::fs;
use std::path::Path as FsPath;
use syn::{
    parse_macro_input,
    token::{Dot, Paren},
    visit_mut::VisitMut,
    Expr, ExprCall, ExprClosure, ExprLit, ExprMethodCall, ExprPath, ItemFn, Lit, LitStr, Path,
    PathSegment,
};

/// A macro that wraps your main function with the necessary boilerplate to run a Ruxt application.
/// The main function should be an async function that returns a `std::io::Result<()>`.
/// The macro will generate the necessary code to run an Actix Web server with the routes defined in the `src/pages` folder.
///
/// # Example
/// ```rust
/// #[ruxt::main]
/// async fn main() -> std::io::Result<()> {
///    let test_data = "Hello, World!";
///    HttpServer::new(move || App::new().app_data(test_data.to_string()))
///    .bind(("0.0.0.0", 8080))?
///    .run()
///    .await
/// }
#[proc_macro_attribute]
pub fn main(_args: TokenStream, item: TokenStream) -> TokenStream {
    process_main(item)
}

pub(crate) fn process_main(item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let mut input = parse_macro_input!(item as ItemFn);

    let mut visitor = ExprVisitor;
    visitor.visit_item_fn_mut(&mut input);

    // Return the modified function as a token stream
    quote!(
    use actix_web::route;

    #[actix_web::main]
    #input
    )
    .into()
}

/// Empty struct that implements the `VisitMut` trait
struct ExprVisitor;

impl VisitMut for ExprVisitor {
    /// The actix web server is always created with this syntax:
    /// ```rust
    /// HttpServer::new(move || App::new()))
    /// ```
    ///
    /// The goal of this function is to do the following:
    /// * Loop over each expression in the current function, looking for one with a path
    ///    segment that matches `HttpServer::new`
    /// * Check if the first argument is a closure
    /// * Generate the necessary route methods for each route in the `src/pages` folder
    /// * Replace the closure with a new closure that contains the generated route methods
    fn visit_expr_call_mut(&mut self, call: &mut ExprCall) {
        if let Expr::Path(ref path) = *call.func {
            if path
                .path
                .segments
                .iter()
                .any(|segment| segment.ident == "HttpServer")
            {
                for segment in &path.path.segments {
                    // Check if the call is for the `new` method
                    if segment.ident == "new" {
                        // Check if the first argument is a closure
                        if let Some(closure) = call.args.first_mut() {
                            if let Expr::Closure(closure_expr) = closure {
                                let body = *closure_expr.body.clone();
                                let routes = generate_routes();
                                let mut current_body = body.clone();

                                for route in routes {
                                    let method_call =
                                        generate_route_method_call(current_body, route);
                                    current_body = method_call.clone();
                                    *closure = Expr::Closure(ExprClosure {
                                        attrs: vec![],
                                        asyncness: None,
                                        movability: None,
                                        capture: Some(Default::default()),
                                        or1_token: Default::default(),
                                        inputs: Default::default(),
                                        or2_token: Default::default(),
                                        output: syn::ReturnType::Default,
                                        body: Box::new(method_call),
                                        lifetimes: None,
                                        constness: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        };

        // If this is not the call expression we're looking for,
        // continue recursively searching
        syn::visit_mut::visit_expr_call_mut(self, call);
    }
}

// Possible options:
//
// Closure:
//  HttpServer::new(move || App::new().app_data(test_data.to_string()))
//
// Function:
// HttpServer::new(app)

fn locate_app_new_expr(expr: &Expr) -> Option<&Expr> {
    // match expr {}

    None
}

/// Generates the method call for a given route location
fn generate_route_method_call(receiver: Expr, route_path: Vec<String>) -> Expr {
    let segments = generate_route_segments(route_path.clone());

    let path = Path {
        leading_colon: None,
        segments: segments.into_iter().collect(),
    };

    let path_expr = Expr::Path(ExprPath {
        attrs: Default::default(),
        qself: None,
        path,
    });

    let path_method_fn_expr = Expr::MethodCall(generate_web_get_to_path(path_expr));

    let mut route_vec = route_path
        .iter()
        .map(|r| {
            if r == "index" {
                "".to_string()
            } else {
                r.to_string()
            }
        })
        .collect::<Vec<String>>();

    if let Some(last) = route_vec.last_mut() {
        if last.is_empty() {
            // Remove the last element if it's empty
            route_vec.pop();
        }
    }

    let route = format!("/{}", route_vec.join("/"));

    let route_expr = Expr::Lit(ExprLit {
        attrs: Default::default(),
        lit: Lit::Str(LitStr::new(&route, proc_macro2::Span::call_site())),
    });

    let args = vec![route_expr, path_method_fn_expr];

    let method: syn::Ident = syn::Ident::new("route", proc_macro2::Span::call_site());

    Expr::MethodCall(ExprMethodCall {
        attrs: Default::default(),
        receiver: Box::new(receiver),
        method,
        turbofish: None,
        args: args.into_iter().collect(),
        dot_token: Dot::default(),
        paren_token: Paren::default(),
    })
}

/// Generates the path segments for a given route location
fn generate_route_segments(route_path: Vec<String>) -> Vec<PathSegment> {
    let route_path = vec!["pages".to_string()]
        .into_iter()
        .chain(route_path.into_iter())
        .collect::<Vec<String>>();

    route_path
        .iter()
        .map(|segment| PathSegment {
            ident: syn::Ident::new(segment, proc_macro2::Span::call_site()),
            arguments: Default::default(),
        })
        .chain(std::iter::once(PathSegment {
            ident: syn::Ident::new("page", proc_macro2::Span::call_site()),
            arguments: Default::default(),
        }))
        .collect()
}

/// Generates the routes for the application from the `src/pages` folder
fn generate_routes() -> Vec<Vec<String>> {
    let mut routes = Vec::new();
    let pages_path = FsPath::new("src/pages");

    // Check if src/pages folder exists
    if !pages_path.exists() || !pages_path.is_dir() {
        panic!("The src/pages folder does not exist!");
    }

    // Recursively loop through each folder in src/pages
    visit_dirs(&pages_path, &mut Vec::new(), &mut routes);

    routes
}

fn visit_dirs(dir: &FsPath, current_dir: &mut Vec<String>, result: &mut Vec<Vec<String>>) {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                if path.eq(FsPath::new("src/pages")) {
                    continue;
                }

                current_dir.push(path.to_str().unwrap().to_string());

                visit_dirs(&path, current_dir, result);
            } else {
                if let Some(ext) = path.extension() {
                    if ext == "rs" {
                        let route = path.strip_prefix("src/pages").unwrap().with_extension("");
                        let route_str = route.to_str().unwrap();

                        if route_str != "src/pages" {
                            let route_components = route_str.split('/').collect::<Vec<&str>>();

                            if !route_components.iter().any(|s| s == &"mod") {
                                let route_components = route_components
                                    .iter()
                                    .map(|s| s.to_string())
                                    .collect::<Vec<String>>();
                                result
                                    .push(route_components.iter().map(|s| s.to_string()).collect());
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Generates the `web::get().to()` method call
fn generate_web_get_to_path(route_function_path: Expr) -> ExprMethodCall {
    let method: syn::Ident = syn::Ident::new("to", proc_macro2::Span::call_site());
    let args = vec![route_function_path];

    ExprMethodCall {
        attrs: Default::default(),
        receiver: Box::new(Expr::Call(ExprCall {
            paren_token: Paren::default(),
            args: Default::default(),
            attrs: Default::default(),
            func: Box::new(Expr::Path(ExprPath {
                attrs: Default::default(),
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments: vec![
                        PathSegment {
                            ident: syn::Ident::new("actix_web", proc_macro2::Span::call_site()),
                            arguments: Default::default(),
                        },
                        PathSegment {
                            ident: syn::Ident::new("web", proc_macro2::Span::call_site()),
                            arguments: Default::default(),
                        },
                        PathSegment {
                            ident: syn::Ident::new("get", proc_macro2::Span::call_site()),
                            arguments: Default::default(),
                        },
                    ]
                    .into_iter()
                    .collect(),
                },
            })),
        })),
        method,
        turbofish: None,
        args: args.into_iter().collect(),
        dot_token: Dot::default(),
        paren_token: Paren::default(),
    }
}
