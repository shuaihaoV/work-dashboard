use axum::body::Body;
use axum::http::header::{self, HeaderValue};
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../frontend/dist"]
struct FrontendAssets;

pub async fn serve_spa(uri: Uri, base_path: &str) -> Response {
    let path = normalize_path(uri.path(), base_path);
    if path != "index.html" {
        if let Some(response) = load_embedded_response(&path) {
            return response;
        }

        if !path.contains('.') {
            if let Some(index_response) = load_index_response(base_path) {
                return index_response;
            }
        }

        return (StatusCode::NOT_FOUND, "asset not found").into_response();
    }

    if let Some(index_response) = load_index_response(base_path) {
        return index_response;
    }

    (StatusCode::NOT_FOUND, "frontend assets not found").into_response()
}

fn normalize_path(path: &str, base_path: &str) -> String {
    let without_base = if base_path != "/" {
        path.strip_prefix(base_path).unwrap_or(path)
    } else {
        path
    };

    let trimmed = without_base.trim_start_matches('/');
    if trimmed.is_empty() {
        "index.html".to_string()
    } else {
        trimmed.to_string()
    }
}

fn load_embedded_response(path: &str) -> Option<Response> {
    let file = FrontendAssets::get(path)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut response = Response::new(Body::from(file.data.into_owned()));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref()).ok()?,
    );
    Some(response)
}

fn load_index_response(base_path: &str) -> Option<Response> {
    let file = FrontendAssets::get("index.html")?;
    let mut html = String::from_utf8(file.data.into_owned()).ok()?;

    let path_prefix = if base_path == "/" { "" } else { base_path };
    if !path_prefix.is_empty() {
        let src_from = "src=\"/assets/";
        let href_from = "href=\"/assets/";

        let src_to = format!("src=\"{path_prefix}/assets/");
        let href_to = format!("href=\"{path_prefix}/assets/");

        html = html.replace(src_from, &src_to);
        html = html.replace(href_from, &href_to);
    }

    let base_script =
        format!("<script>window.__WORK_DASHBOARD_BASE_PATH__=\"{base_path}\";</script>");
    if let Some(head_end) = html.find("</head>") {
        html.insert_str(head_end, &base_script);
    } else {
        html.push_str(&base_script);
    }

    let mut response = Response::new(Body::from(html));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    Some(response)
}
