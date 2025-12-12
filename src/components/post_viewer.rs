use crate::components::router::Route;
use gloo_net::http::Request;
use gloo_utils::document;
use pulldown_cmark::{CowStr, Event, Parser, Tag, html};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PostProps {
    pub slug: String,
}

#[function_component(PostViewer)]
pub fn post_viewer(props: &PostProps) -> Html {
    let content = use_state(|| String::new());
    let slug = props.slug.clone();

    {
        let content = content.clone();
        let slug = slug.clone();
        use_effect_with(slug.clone(), move |_| {
            spawn_local(async move {
                let url = format!("posts/{}/{}.md", slug, slug);
                if let Ok(response) = Request::get(&url).send().await {
                    if let Ok(text) = response.text().await {
                        let mut options = pulldown_cmark::Options::empty();
                        options.insert(pulldown_cmark::Options::ENABLE_TABLES);

                        let parser = Parser::new_ext(&text, options);
                        let parser = parser.map(|event| match event {
                            Event::Start(Tag::Image {
                                link_type,
                                dest_url,
                                title,
                                id,
                            }) => {
                                if !dest_url.starts_with("http") && !dest_url.starts_with("/") {
                                    let new_dest = format!("posts/{}/{}", slug, dest_url);
                                    Event::Start(Tag::Image {
                                        link_type,
                                        dest_url: CowStr::from(new_dest),
                                        title,
                                        id,
                                    })
                                } else {
                                    Event::Start(Tag::Image {
                                        link_type,
                                        dest_url,
                                        title,
                                        id,
                                    })
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

    let div = document().create_element("div").unwrap();
    div.set_inner_html(&*content);
    let node = Html::VRef(div.into());

    html! {
        <div class="post-container">
            <nav>
                <Link<Route> to={Route::Home}>{"Back to Home"}</Link<Route>>
            </nav>
            <article class="markdown-body">
                { node }
            </article>
        </div>
    }
}
