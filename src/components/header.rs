use yew::prelude::*;

#[function_component(Header)]
pub fn header() -> Html {
    html! {
            <header>
                <h1 class="text-5xl font-bold text-blue-600">{ "Blog do luis" }</h1>
            </header>
    }
}
