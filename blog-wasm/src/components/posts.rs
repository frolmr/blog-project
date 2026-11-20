use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::api::{self, ApiError};
use crate::components::inputs::{on_input, on_textarea_input};
use crate::format::format_time;
use crate::models::Post;

#[derive(Properties, PartialEq)]
pub struct PostCardProps {
    pub post: Post,
    pub current_user_id: Option<i64>,
    pub token: String,
    pub on_updated: Callback<Post>,
    pub on_deleted: Callback<i64>,
    pub on_error: Callback<ApiError>,
}

#[function_component(PostCard)]
pub fn post_card(props: &PostCardProps) -> Html {
    let editing = use_state(|| false);
    let draft_title = use_state(|| props.post.title.clone());
    let draft_content = use_state(|| props.post.content.clone());
    let busy = use_state(|| false);
    let local_error = use_state(|| None::<String>);

    let is_author = props.current_user_id == Some(props.post.author_id);

    let start_edit = {
        let editing = editing.clone();
        let draft_title = draft_title.clone();
        let draft_content = draft_content.clone();
        let local_error = local_error.clone();
        let title = props.post.title.clone();
        let content = props.post.content.clone();

        Callback::from(move |_: MouseEvent| {
            draft_title.set(title.clone());
            draft_content.set(content.clone());
            local_error.set(None);
            editing.set(true);
        })
    };

    let cancel_edit = {
        let editing = editing.clone();
        let local_error = local_error.clone();

        Callback::from(move |_: MouseEvent| {
            local_error.set(None);
            editing.set(false);
        })
    };

    let onsubmit = {
        let editing = editing.clone();
        let draft_title = draft_title.clone();
        let draft_content = draft_content.clone();
        let busy = busy.clone();
        let local_error = local_error.clone();
        let token = props.token.clone();
        let id = props.post.id;
        let on_updated = props.on_updated.clone();
        let on_error = props.on_error.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();

            let title = draft_title.trim().to_owned();
            let content = draft_content.trim().to_owned();

            if title.is_empty() || content.is_empty() {
                local_error.set(Some("Title and content must not be empty.".to_owned()));
                return;
            }

            local_error.set(None);
            busy.set(true);

            let editing = editing.clone();
            let busy = busy.clone();
            let token = token.clone();
            let on_updated = on_updated.clone();
            let on_error = on_error.clone();

            spawn_local(async move {
                match api::update_post(&token, id, Some(title), Some(content)).await {
                    Ok(post) => {
                        busy.set(false);
                        editing.set(false);
                        on_updated.emit(post);
                    }
                    Err(error) => {
                        busy.set(false);
                        on_error.emit(error);
                    }
                }
            });
        })
    };

    let ondelete = {
        let busy = busy.clone();
        let token = props.token.clone();
        let id = props.post.id;
        let on_deleted = props.on_deleted.clone();
        let on_error = props.on_error.clone();

        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            busy.set(true);

            let busy = busy.clone();
            let token = token.clone();
            let on_deleted = on_deleted.clone();
            let on_error = on_error.clone();

            spawn_local(async move {
                match api::delete_post(&token, id).await {
                    Ok(()) => {
                        busy.set(false);
                        on_deleted.emit(id);
                    }
                    Err(error) => {
                        busy.set(false);
                        on_error.emit(error);
                    }
                }
            });
        })
    };

    let view = html! {
        <>
            <div
                class="card-body"
                onclick={is_author.then(|| start_edit.clone())}
            >
                <div class="d-flex align-items-center flex-wrap gap-2 mb-2">
                    <h3 class="h5 mb-0">{ &props.post.title }</h3>
                    if is_author {
                        <span class="badge rounded-pill text-bg-primary">{ "your post" }</span>
                    }
                </div>
                <p class="mb-0 text-break" style="white-space: pre-wrap;">
                    { &props.post.content }
                </p>
                <div class="d-flex flex-wrap gap-2 gap-sm-3 small text-body-secondary mt-3">
                    <span>{ format!("author #{}", props.post.author_id) }</span>
                    <span>{ format_time(&props.post.created_at) }</span>
                    if props.post.updated_at != props.post.created_at {
                        <span>{ format!("edited {}", format_time(&props.post.updated_at)) }</span>
                    }
                </div>
            </div>

            if is_author {
                <div class="card-footer d-flex justify-content-end gap-2">
                    <button
                        type="button"
                        class="btn btn-sm btn-outline-secondary"
                        onclick={start_edit}
                        disabled={*busy}
                    >
                        { "Edit" }
                    </button>
                    <button
                        type="button"
                        class="btn btn-sm btn-outline-danger"
                        onclick={ondelete}
                        disabled={*busy}
                    >
                        { if *busy { "Deleting…" } else { "Delete" } }
                    </button>
                </div>
            }
        </>
    };

    let editor = html! {
        <form class="card-body d-grid gap-3" onsubmit={onsubmit}>
            <label class="d-block">
                <span class="form-label">{ "Title" }</span>
                <input
                    class="form-control"
                    type="text"
                    value={(*draft_title).clone()}
                    oninput={on_input(draft_title.clone())}
                    disabled={*busy}
                />
            </label>

            <label class="d-block">
                <span class="form-label">{ "Content" }</span>
                <textarea
                    class="form-control"
                    rows="4"
                    value={(*draft_content).clone()}
                    oninput={on_textarea_input(draft_content.clone())}
                    disabled={*busy}
                />
            </label>

            if let Some(message) = local_error.as_ref() {
                <p class="text-danger small mb-0" role="alert">{ message }</p>
            }

            <div class="d-flex gap-2">
                <button type="submit" class="btn btn-primary" disabled={*busy}>
                    { if *busy { "Saving…" } else { "Save changes" } }
                </button>
                <button
                    type="button"
                    class="btn btn-outline-secondary"
                    onclick={cancel_edit}
                    disabled={*busy}
                >
                    { "Cancel" }
                </button>
            </div>
        </form>
    };

    html! {
        <article class={classes!("card", "shadow-sm", is_author.then_some("border-primary"))}>
            if is_author && *editing {
                { editor }
            } else {
                { view }
            }
        </article>
    }
}
