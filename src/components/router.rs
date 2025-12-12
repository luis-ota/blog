use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq, Debug)]
pub enum Route {
    #[at("/")]
    Home,

    #[at("/post/:slug")]
    Post { slug: String },

    #[not_found]
    #[at("/404")]
    NotFound,
}
