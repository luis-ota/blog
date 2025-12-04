use gloo_net::http::Request;
use pulldown_cmark::{Parser, html, Event, Tag, CowStr};
use serde::Deserialize;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/post/:slug")]
    Post { slug: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[derive(Deserialize, Clone, PartialEq)]
struct PostMeta {
    slug: String,
    title: String,
}

#[function_component(Home)]
fn home() -> Html {
    let posts = use_state(|| vec![]);
    {
        let posts = posts.clone();
        use_effect_with((), move |_| {
            let posts = posts.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(response) = Request::get("/posts.json").send().await {
                    if let Ok(fetched_posts) = response.json::<Vec<PostMeta>>().await {
                        posts.set(fetched_posts);
                    }
                }
            });
            || ()
        });
    }

    html! {
        <div class="container">
            <header>
                <h1 class="text-5xl font-bold text-blue-600">{ "My Awesome Blog" }</h1>
            </header>
            <main>
                <ul class="post-list">
                    { for posts.iter().map(|post| html! {
                        <li key={post.slug.clone()}>
                            <Link<Route> to={Route::Post { slug: post.slug.clone() }} classes="post-link">
                                { &post.title }
                            </Link<Route>>
                        </li>
                    }) }
                </ul>
            </main>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct PostProps {
    slug: String,
}

#[function_component(PostViewer)]
fn post_viewer(props: &PostProps) -> Html {
    let content = use_state(|| String::new());
    let slug = props.slug.clone();

    {
        let content = content.clone();
        let slug = slug.clone();
        use_effect_with(slug.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/posts/{}/{}.md", slug, slug);
                if let Ok(response) = Request::get(&url).send().await {
                    if let Ok(text) = response.text().await {
                        let mut options = pulldown_cmark::Options::empty();
                        options.insert(pulldown_cmark::Options::ENABLE_TABLES);
                        let parser = Parser::new_ext(&text, options);
                        let parser = parser.map(|event| match event {
                            Event::Start(Tag::Image { link_type, dest_url, title, id }) => {
                                if !dest_url.starts_with("http") && !dest_url.starts_with("/") {
                                    let new_dest = format!("/posts/{}/{}", slug, dest_url);
                                    Event::Start(Tag::Image {
                                        link_type,
                                        dest_url: CowStr::from(new_dest),
                                        title,
                                        id,
                                    })
                                } else {
                                    Event::Start(Tag::Image { link_type, dest_url, title, id })
                                }
                            }
                            _ => event,
                        });
                        let mut html_output = String::new();
                        html::push_html(&mut html_output, parser);
                        content.set(html_output);
                    }
                }
            });
            || ()
        });
    }

    let div = gloo_utils::document().create_element("div").unwrap();
    div.set_inner_html(&*content);
    let node = Html::VRef(div.into());

    html! {
        <div class="post-container">
            <nav>
                <Link<Route> to={Route::Home}>{"Back to Home" }</Link<Route>>
            </nav>
            <article class="markdown-body">
                { node }
            </article>
        </div>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Post { slug } => html! { <PostViewer slug={slug} /> },
        Route::NotFound => html! { <h1>{ "404 Not Found" }</h1> },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
