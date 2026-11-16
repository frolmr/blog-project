use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <main class="app">
            <h1>{ "Blog" }</h1>
            <p>{ "Yew + WebAssembly frontend is up and running." }</p>
        </main>
    }
}

#[wasm_bindgen(start)]
pub fn run() {
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
