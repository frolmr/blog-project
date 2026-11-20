use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::api::{self, ApiError};
use crate::components::inputs::{on_input, on_textarea_input};
use crate::models::Post;

#[derive(Properties, PartialEq)]
pub struct CreatePostProps {
    pub token: String,
    pub on_created: Callback<Post>,
    pub on_error: Callback<ApiError>,
}

#[function_component(CreatePostForm)]
pub fn create_post_form(props: &CreatePostProps) -> Html {
    let title = use_state(String::new);
    let content = use_state(String::new);
    let busy = use_state(|| false);
    let local_error = use_state(|| None::<String>);

    let onsubmit = {
        let title = title.clone();
        let content = content.clone();
        let busy = busy.clone();
        let local_error = local_error.clone();
        let token = props.token.clone();
        let on_created = props.on_created.clone();
        let on_error = props.on_error.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();

            let title_value = title.trim().to_owned();
            let content_value = content.trim().to_owned();

            if title_value.is_empty() || content_value.is_empty() {
                local_error.set(Some("Title and content must not be empty.".to_owned()));
                return;
            }

            local_error.set(None);
            busy.set(true);

            let title = title.clone();
            let content = content.clone();
            let busy = busy.clone();
            let token = token.clone();
            let on_created = on_created.clone();
            let on_error = on_error.clone();

            spawn_local(async move {
                match api::create_post(&token, &title_value, &content_value).await {
                    Ok(post) => {
                        busy.set(false);
                        title.set(String::new());
                        content.set(String::new());
                        on_created.emit(post);
                    }
                    Err(error) => {
                        busy.set(false);
                        on_error.emit(error);
                    }
                }
            });
        })
    };

    html! {
        <form class="card shadow-sm" onsubmit={onsubmit}>
            <div class="card-body d-grid gap-3">
                <h2 class="h4 card-title mb-0">{ "Publish a post" }</h2>

                <label class="d-block">
                    <span class="form-label">{ "Title" }</span>
                    <input
                        class="form-control"
                        type="text"
                        value={(*title).clone()}
                        oninput={on_input(title.clone())}
                        disabled={*busy}
                    />
                </label>

                <label class="d-block">
                    <span class="form-label">{ "Content" }</span>
                    <textarea
                        class="form-control"
                        rows="4"
                        value={(*content).clone()}
                        oninput={on_textarea_input(content.clone())}
                        disabled={*busy}
                    />
                </label>

                if let Some(message) = local_error.as_ref() {
                    <p class="text-danger small mb-0" role="alert">{ message }</p>
                }

                <div>
                    <button type="submit" class="btn btn-primary" disabled={*busy}>
                        { if *busy { "Publishing…" } else { "Publish" } }
                    </button>
                </div>
            </div>
        </form>
    }
}
