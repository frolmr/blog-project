use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::api::{self, ApiError};
use crate::auth;
use crate::components::auth_forms::{LoginForm, RegisterForm};
use crate::components::create_post::CreatePostForm;
use crate::components::posts::PostCard;
use crate::models::{AuthResponse, Post};
use crate::storage;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Session {
    token: String,
    user_id: i64,
    username: String,
}

fn restore_session() -> Option<Session> {
    let token = storage::get_token_from_storage()?;

    match auth::claims_from_token(&token) {
        Some(claims) => Some(Session {
            token,
            user_id: claims.user_id,
            username: claims.username,
        }),
        None => {
            storage::clear_token_from_storage();
            None
        }
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let session = use_state(restore_session);
    let posts = use_state(Vec::<Post>::new);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);

    let reload = {
        let posts = posts.clone();
        let loading = loading.clone();
        let error = error.clone();

        Callback::from(move |_: ()| {
            loading.set(true);

            let posts = posts.clone();
            let loading = loading.clone();
            let error = error.clone();

            spawn_local(async move {
                match api::list_posts().await {
                    Ok(page) => {
                        posts.set(page.posts);
                        error.set(None);
                    }
                    Err(api_error) => error.set(Some(api_error.to_string())),
                }
                loading.set(false);
            });
        })
    };

    {
        let reload = reload.clone();
        use_effect_with((), move |_| {
            reload.emit(());
            || ()
        });
    }

    let refresh = {
        let reload = reload.clone();
        Callback::from(move |_: MouseEvent| reload.emit(()))
    };

    let on_authenticated = {
        let session = session.clone();
        let error = error.clone();
        let reload = reload.clone();

        Callback::from(move |auth: AuthResponse| {
            let AuthResponse { user, access_token } = auth;

            if let Err(message) = storage::save_token_to_storage(&access_token) {
                error.set(Some(message));
                return;
            }

            session.set(Some(Session {
                token: access_token,
                user_id: user.id,
                username: user.username,
            }));
            error.set(None);
            reload.emit(());
        })
    };

    let on_error = {
        let session = session.clone();
        let error = error.clone();

        Callback::from(move |api_error: ApiError| {
            if api_error.status == 401 {
                storage::clear_token_from_storage();
                session.set(None);
            }

            error.set(Some(api_error.to_string()));
        })
    };

    let logout = {
        let session = session.clone();
        let error = error.clone();

        Callback::from(move |_: MouseEvent| {
            storage::clear_token_from_storage();
            session.set(None);
            error.set(None);
        })
    };

    let on_created = {
        let posts = posts.clone();

        Callback::from(move |post: Post| {
            let mut next = (*posts).clone();
            next.insert(0, post);
            posts.set(next);
        })
    };

    let on_updated = {
        let posts = posts.clone();

        Callback::from(move |updated: Post| {
            let next = posts
                .iter()
                .map(|post| {
                    if post.id == updated.id {
                        updated.clone()
                    } else {
                        post.clone()
                    }
                })
                .collect();

            posts.set(next);
        })
    };

    let on_deleted = {
        let posts = posts.clone();

        Callback::from(move |id: i64| {
            let next = posts.iter().filter(|post| post.id != id).cloned().collect();

            posts.set(next);
        })
    };

    let current_user_id = session.as_ref().map(|session| session.user_id);
    let session_token = session
        .as_ref()
        .map(|session| session.token.clone())
        .unwrap_or_default();

    html! {
        <div class="container py-4 py-lg-5">
            <header class="d-flex flex-column flex-sm-row align-items-sm-center justify-content-between gap-3 mb-4">
                <div class="d-flex align-items-center gap-3">
                    <span class="d-inline-flex align-items-center justify-content-center rounded-3 bg-primary bg-gradient text-white fw-bold fs-5 px-3 py-2">
                        { "B" }
                    </span>
                    <div>
                        <h1 class="fs-3 fw-semibold mb-0">{ "Blog" }</h1>
                        <p class="text-body-secondary small mb-0">{ "Yew + WebAssembly" }</p>
                    </div>
                </div>

                <div class="d-flex align-items-center gap-2">
                    if let Some(session) = session.as_ref() {
                        <span class="badge rounded-pill text-success-emphasis bg-success-subtle border border-success-subtle">
                            { format!("Logged in as {}", session.username) }
                        </span>
                        <button class="btn btn-outline-secondary btn-sm" onclick={logout}>
                            { "Log out" }
                        </button>
                    } else {
                        <span class="badge rounded-pill text-body-secondary bg-body-secondary border">
                            { "Not logged in" }
                        </span>
                    }
                </div>
            </header>

            <main class="d-grid gap-3">
                if let Some(message) = error.as_ref() {
                    <div class="alert alert-danger d-flex flex-column flex-sm-row align-items-baseline gap-2 mb-0" role="alert">
                        <strong class="text-uppercase small">{ "Error" }</strong>
                        <span>{ message.clone() }</span>
                    </div>
                }

                if let Some(session) = session.as_ref() {
                    <CreatePostForm
                        token={session.token.clone()}
                        on_created={on_created.clone()}
                        on_error={on_error.clone()}
                    />
                } else {
                    <section class="row g-3">
                        <div class="col-12 col-lg-6">
                            <RegisterForm
                                on_authenticated={on_authenticated.clone()}
                                on_error={on_error.clone()}
                            />
                        </div>
                        <div class="col-12 col-lg-6">
                            <LoginForm
                                on_authenticated={on_authenticated.clone()}
                                on_error={on_error.clone()}
                            />
                        </div>
                    </section>
                }

                <section class="card shadow-sm">
                    <div class="card-body">
                        <div class="d-flex align-items-start justify-content-between gap-3 mb-3">
                            <div>
                                <h2 class="h4 mb-1">{ "Posts" }</h2>
                                <p class="text-body-secondary small mb-0">
                                    { format!("{} shown", posts.len()) }
                                </p>
                            </div>
                            <button
                                type="button"
                                class="btn btn-outline-secondary btn-sm"
                                onclick={refresh}
                                disabled={*loading}
                            >
                                { if *loading { "Loading…" } else { "Refresh" } }
                            </button>
                        </div>

                        if *loading {
                            <p class="text-body-secondary mb-0">{ "Loading posts…" }</p>
                        } else if posts.is_empty() {
                            <p class="text-body-secondary mb-0">
                                { "No posts yet. Be the first to publish one." }
                            </p>
                        } else {
                            <div class="d-grid gap-3">
                                for post in posts.iter() {
                                    <PostCard
                                        key={post.id}
                                        post={post.clone()}
                                        current_user_id={current_user_id}
                                        token={session_token.clone()}
                                        on_updated={on_updated.clone()}
                                        on_deleted={on_deleted.clone()}
                                        on_error={on_error.clone()}
                                    />
                                }
                            </div>
                        }
                    </div>
                </section>
            </main>
        </div>
    }
}
