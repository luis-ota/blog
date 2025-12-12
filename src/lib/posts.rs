use crate::models::post::PostMeta;
use gloo_net::http::Request;

/// Fetch the posts index (`posts.json`) and return a vector of `PostMeta`.
///
/// Note: in a browser / WASM environment we cannot read the server filesystem
/// directly. The client should request a `posts.json` file (generated server-side
/// or placed in the `dist`/static assets) which contains the list of posts.
///
/// This function returns an empty Vec on failure.
pub(crate) async fn fetch_posts() -> Vec<PostMeta> {
    match Request::get("posts.json").send().await {
        Ok(resp) => match resp.json::<Vec<PostMeta>>().await {
            Ok(posts) => posts,
            Err(_) => Vec::new(),
        },
        Err(_) => Vec::new(),
    }
}
