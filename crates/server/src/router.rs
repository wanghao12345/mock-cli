use std::sync::Arc;

use axum::{Json, Router, routing::{MethodRouter, delete, get, options, patch, post, put}};
use mock_cli_core::generate_mock;
use openapiv3::{OpenAPI, PathItem};

fn make_handler(
    spec: Arc<OpenAPI>,
    path: String,
    method: &'static str,
) -> impl Fn() -> std::future::Ready<Json<serde_json::Value>> + Clone + Send + Sync + 'static {
    move || {
        let s = spec.clone();
        let p = path.clone();
        let m = method;
        std::future::ready(Json(generate_mock(&s, &p, m)))
    }
}

fn merge_method(mr: Option<MethodRouter>, new: MethodRouter) -> Option<MethodRouter> {
    Some(match mr {
        Some(existing) => existing.merge(new),
        None => new,
    })
}

fn register_methods(
    spec: Arc<OpenAPI>,
    path: String,
    item: &PathItem,
) -> Option<MethodRouter> {
    let mut methods: Option<MethodRouter> = None;

    if item.get.is_some() {
        methods = merge_method(methods, get(make_handler(spec.clone(), path.clone(), "GET")));
    }
    if item.post.is_some() {
        methods = merge_method(methods, post(make_handler(spec.clone(), path.clone(), "POST")));
    }
    if item.put.is_some() {
        methods = merge_method(methods, put(make_handler(spec.clone(), path.clone(), "PUT")));
    }
    if item.delete.is_some() {
        methods = merge_method(methods, delete(make_handler(spec.clone(), path.clone(), "DELETE")));
    }
    if item.options.is_some() {
        methods = merge_method(methods, options(make_handler(spec.clone(), path.clone(), "OPTIONS")));
    }
    if item.patch.is_some() {
        methods = merge_method(methods, patch(make_handler(spec.clone(), path.clone(), "PATCH")));
    }
    if item.head.is_some() {
        methods = merge_method(methods, axum::routing::head(make_handler(spec.clone(), path.clone(), "HEAD")));
    }
    if item.trace.is_some() {
        methods = merge_method(methods, axum::routing::trace(make_handler(spec.clone(), path.clone(), "TRACE")));
    }

    methods
}

/// Build the router for the mock server.
pub fn build_router(spec: Arc<OpenAPI>) -> Router {
    let mut app = Router::new();
    for (path, item) in spec.paths.paths.iter() {
        let Some(item) = item.as_item() else {
            continue;
        };
        if let Some(method_router) = register_methods(spec.clone(), path.clone(), item) {
            app = app.route(path, method_router);
        }
    }
    app
}
