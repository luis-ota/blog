use gloo_net::http::Request;
use gloo_utils::document;
use pulldown_cmark::{Options, Parser, html};
use serde::Deserialize;
use std::fs;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::use_navigator;

use crate::{components::router::Route, models::post::PostMeta};

#[function_component(Home)]
pub fn home() -> Html {
    let posts = use_state(|| Vec::<PostMeta>::new());
    // currently active folder/post slug
    let active = use_state(|| Option::<String>::None);
    // preview HTML for the active folder
    let preview_html = use_state(|| String::new());
    // loading state for preview
    let loading = use_state(|| false);

    let navigator = use_navigator().unwrap();

    {
        let posts = posts.clone();
        use_effect_with((), move |_| {
            let posts = posts.clone();
            spawn_local(async move {
                let fetched = crate::lib::posts::fetch_posts().await;
                posts.set(fetched);
            });
            || ()
        });
    }

    let fetch_preview = {
        let preview_html_handle = preview_html.clone();
        let loading_handle = loading.clone();
        Callback::from(move |slug: String| {
            // clone handles inside the closure so this closure owns its copies
            let preview_html = preview_html_handle.clone();
            let loading = loading_handle.clone();
            loading.set(true);
            spawn_local(async move {
                let url = format!("posts/{}/{}.md", slug, slug);
                match Request::get(&url).send().await {
                    Ok(resp) => match resp.text().await {
                        Ok(text) => {
                            let mut options = Options::empty();
                            options.insert(Options::ENABLE_TABLES);
                            let parser = Parser::new_ext(&text, options);

                            let mut html_output = String::new();
                            html::push_html(&mut html_output, parser);

                            // take the first paragraph or up to the first 600 chars
                            let preview = html_output
                                .split("</p>")
                                .next()
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| {
                                    let mut t = html_output;
                                    t.truncate(600);
                                    t
                                });

                            preview_html.set(preview);
                        }
                        Err(_) => {
                            preview_html.set("Erro ao carregar preview.".to_string());
                        }
                    },
                    Err(_) => {
                        preview_html.set("Erro ao carregar preview.".to_string());
                    }
                }
                loading.set(false);
            });
        })
    };

    // Click handler: if clicking already active -> open post page; otherwise set active and fetch preview
    let on_tab_click = {
        let active = active.clone();
        let fetch_preview = fetch_preview.clone();
        // clone a handle specifically for this closure so we don't move the original `preview_html`
        let preview_html_for_onclick = preview_html.clone();
        Callback::from(move |slug: String| {
            let is_same = active.as_ref().map(|s| s == &slug).unwrap_or(false);
            if is_same {
                // handled in the button closure below where navigator is available
            } else {
                active.set(Some(slug.clone()));
                preview_html_for_onclick.set(String::new());
                fetch_preview.emit(slug);
            }
        })
    };

    // Build preview node from string safely by creating a DOM element and converting to VRef
    let preview_node = {
        if preview_html.is_empty() {
            Html::default()
        } else {
            let div = document().create_element("div").unwrap();
            div.set_inner_html(&*preview_html);
            Html::VRef(div.into())
        }
    };

    // Tabs rendering: use CSS custom property --i for stacking offset (kept for potential CSS use)
    let tabs = posts
        .iter()
        .enumerate()
        .map(|(i, post)| {
            let slug = post.slug.clone();
            let title = post.title.clone();
            // two clones so we can keep `slug` for local checks while moving one clone into the closure
            let key_slug = slug.clone();
            let onclick_slug = slug.clone();
            let is_active = active.as_ref().map(|s| s == &slug).unwrap_or(false);
            let idx = i as i32;
            let onclick = {
                let on_tab_click = on_tab_click.clone();
                let navigator = navigator.clone();
                let active = active.clone();
                // move the closure-specific clone into the closure
                let onclick_slug = onclick_slug.clone();
                Callback::from(move |_| {
                    if active.as_ref().map(|s| s == &onclick_slug).unwrap_or(false) {
                        navigator.push(&Route::Post { slug: onclick_slug.clone() });
                    } else {
                        on_tab_click.emit(onclick_slug.clone());
                    }
                })
            };

            // Tailwind-first classes for a folder/tab visual
            let base_classes = "flex items-center gap-3 w-52 h-14 rounded-md px-3 transition-transform duration-300 ease-in-out transform";
            let inactive_classes = "bg-slate-800 text-slate-100 hover:-translate-y-2 hover:scale-105 hover:shadow-lg";
            let active_classes = "-translate-y-3 scale-105 shadow-2xl z-50 bg-white text-slate-900";

            html! {
                <button
                    key={key_slug}
                    class={classes!(
                        base_classes,
                        if is_active { Some(active_classes) } else { Some(inactive_classes) }
                    )}
                    {onclick}
                    aria-pressed={is_active.to_string()}
                    style={format!("--i: {};", idx)}
                    title={title.clone()}
                >
                    <div class="flex-1 text-sm font-semibold truncate">
                        <span class="block">{ title.clone() }</span>
                    </div>
                    <div class="ml-auto bg-slate-700 text-xs px-2 py-1 rounded-r-md">
                        { format!("{}", i + 1) }
                    </div>
                </button>
            }
        })
        .collect::<Html>();

    // Viewer action: open post from viewer button
    let open_post = {
        let active = active.clone();
        let navigator = navigator.clone();
        Callback::from(move |_| {
            if let Some(slug) = active.as_ref() {
                navigator.push(&Route::Post { slug: slug.clone() });
            }
        })
    };

    html! {
        <div class="min-h-screen bg-transparent p-4">
            <div class="grid md:grid-cols-[220px_1fr] grid-cols-1 gap-4 h-[calc(100vh-2rem)]">
                // tabs column
                <aside class="flex md:flex-col flex-row md:overflow-visible overflow-x-auto gap-3 md:items-start items-center" role="tablist">
                    { tabs }
                </aside>

                // viewer area
                <main class="flex items-start justify-center p-4 overflow-auto" aria-live="polite">
                    {
                        if active.is_none() {
                            html! {
                                <div class="max-w-3xl w-full bg-transparent rounded-md p-6 text-slate-200">

                                    <p class="text-lg">
                                        { "Clique em uma pasta para ver o título e o início do post. Clique novamente para abrir a postagem." }
                                    </p>
                                </div>
                            }
                        } else {
                            html! {
                                <div class="w-full max-w-3xl bg-slate-800 rounded-lg p-4 shadow-2xl">
                                    <div class="flex items-center justify-between">
                                        <h3 class="text-xl font-semibold text-cyan-300">
                                            {
                                                active.as_ref().and_then(|slug| {
                                                    posts.iter().find(|p| &p.slug == slug).map(|p| p.title.clone())
                                                }).unwrap_or_else(|| "Post".to_string())
                                            }
                                        </h3>
                                        <div>
                                            <button class="bg-cyan-500 text-white px-3 py-1 rounded-md" onclick={open_post.clone()}>
                                                { "Abrir postagem" }
                                            </button>
                                        </div>
                                    </div>

                                    <div class="mt-4 prose prose-invert max-w-none">
                                        {
                                            if *loading {
                                                html!{ <div class="text-slate-300">{ "Carregando preview..." }</div> }
                                            } else {
                                                preview_node
                                            }
                                        }
                                    </div>
                                </div>
                            }
                        }
                    }
                </main>
            </div>
        </div>
    }
}
