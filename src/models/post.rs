use serde::Deserialize;

#[derive(Clone, Deserialize, PartialEq)]
pub struct PostMeta {
    pub slug: String,
    pub title: String,
}
