//! Registration and login forms.
//!
//! Both components call the browser API and report the result through the
//! callbacks in [`AuthFormProps`].

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::api::{self, ApiError};
use crate::components::inputs::on_input;
use crate::models::AuthResponse;

/// Callbacks shared by the registration and login forms.
#[derive(Properties, PartialEq)]
pub struct AuthFormProps {
    /// Invoked with the server response after a successful request.
    pub on_authenticated: Callback<AuthResponse>,
    /// Invoked when the server or network returns an error.
    pub on_error: Callback<ApiError>,
}

/// Registration form component.
#[function_component(RegisterForm)]
pub fn register_form(props: &AuthFormProps) -> Html {
    let username = use_state(String::new);
    let email = use_state(String::new);
    let password = use_state(String::new);
    let busy = use_state(|| false);
    let local_error = use_state(|| None::<String>);

    let onsubmit = {
        let username = username.clone();
        let email = email.clone();
        let password = password.clone();
        let busy = busy.clone();
        let local_error = local_error.clone();
        let on_authenticated = props.on_authenticated.clone();
        let on_error = props.on_error.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();

            let username_value = username.trim().to_owned();
            let email_value = email.trim().to_owned();
            let password_value = (*password).clone();

            if let Err(message) =
                validate_registration(&username_value, &email_value, &password_value)
            {
                local_error.set(Some(message));
                return;
            }

            local_error.set(None);
            busy.set(true);

            let busy = busy.clone();
            let on_authenticated = on_authenticated.clone();
            let on_error = on_error.clone();

            spawn_local(async move {
                match api::register(&username_value, &email_value, &password_value).await {
                    Ok(auth) => {
                        busy.set(false);
                        on_authenticated.emit(auth);
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
        <form class="card shadow-sm h-100" onsubmit={onsubmit}>
            <div class="card-body d-grid gap-3 align-content-start">
                <div>
                    <h2 class="h4 card-title mb-1">{ "Create an account" }</h2>
                    <p class="card-text text-body-secondary small mb-0">
                        { "Register to publish posts." }
                    </p>
                </div>

                <label class="d-block">
                    <span class="form-label">{ "Username" }</span>
                    <input
                        class="form-control"
                        type="text"
                        autocomplete="username"
                        value={(*username).clone()}
                        oninput={on_input(username.clone())}
                        disabled={*busy}
                    />
                </label>

                <label class="d-block">
                    <span class="form-label">{ "Email" }</span>
                    <input
                        class="form-control"
                        type="email"
                        autocomplete="email"
                        value={(*email).clone()}
                        oninput={on_input(email.clone())}
                        disabled={*busy}
                    />
                </label>

                <label class="d-block">
                    <span class="form-label">{ "Password" }</span>
                    <input
                        class="form-control"
                        type="password"
                        autocomplete="new-password"
                        value={(*password).clone()}
                        oninput={on_input(password.clone())}
                        disabled={*busy}
                    />
                </label>

                if let Some(message) = local_error.as_ref() {
                    <p class="text-danger small mb-0" role="alert">{ message }</p>
                }

                <button type="submit" class="btn btn-primary w-100" disabled={*busy}>
                    { if *busy { "Creating account…" } else { "Register" } }
                </button>
            </div>
        </form>
    }
}

/// Login form component; accepts a username or email.
#[function_component(LoginForm)]
pub fn login_form(props: &AuthFormProps) -> Html {
    let identifier = use_state(String::new);
    let password = use_state(String::new);
    let busy = use_state(|| false);
    let local_error = use_state(|| None::<String>);

    let onsubmit = {
        let identifier = identifier.clone();
        let password = password.clone();
        let busy = busy.clone();
        let local_error = local_error.clone();
        let on_authenticated = props.on_authenticated.clone();
        let on_error = props.on_error.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();

            let identifier_value = identifier.trim().to_owned();
            let password_value = (*password).clone();

            if let Err(message) = validate_login(&identifier_value, &password_value) {
                local_error.set(Some(message));
                return;
            }

            local_error.set(None);
            busy.set(true);

            let busy = busy.clone();
            let on_authenticated = on_authenticated.clone();
            let on_error = on_error.clone();

            spawn_local(async move {
                match api::login(&identifier_value, &password_value).await {
                    Ok(auth) => {
                        busy.set(false);
                        on_authenticated.emit(auth);
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
        <form class="card shadow-sm h-100" onsubmit={onsubmit}>
            <div class="card-body d-grid gap-3 align-content-start">
                <div>
                    <h2 class="h4 card-title mb-1">{ "Log in" }</h2>
                    <p class="card-text text-body-secondary small mb-0">
                        { "Use your username or email." }
                    </p>
                </div>

                <label class="d-block">
                    <span class="form-label">{ "Username or email" }</span>
                    <input
                        class="form-control"
                        type="text"
                        autocomplete="username"
                        value={(*identifier).clone()}
                        oninput={on_input(identifier.clone())}
                        disabled={*busy}
                    />
                </label>

                <label class="d-block">
                    <span class="form-label">{ "Password" }</span>
                    <input
                        class="form-control"
                        type="password"
                        autocomplete="current-password"
                        value={(*password).clone()}
                        oninput={on_input(password.clone())}
                        disabled={*busy}
                    />
                </label>

                if let Some(message) = local_error.as_ref() {
                    <p class="text-danger small mb-0" role="alert">{ message }</p>
                }

                <button type="submit" class="btn btn-primary w-100" disabled={*busy}>
                    { if *busy { "Signing in…" } else { "Log in" } }
                </button>
            </div>
        </form>
    }
}

fn validate_login(identifier: &str, password: &str) -> Result<(), String> {
    if identifier.is_empty() || password.is_empty() {
        return Err("Please fill in every field.".to_owned());
    }

    Ok(())
}

fn validate_registration(username: &str, email: &str, password: &str) -> Result<(), String> {
    if username.is_empty() || email.is_empty() || password.is_empty() {
        return Err("Please fill in every field.".to_owned());
    }

    if username.chars().count() > 32 {
        return Err("Username must be at most 32 characters long.".to_owned());
    }

    if !username
        .chars()
        .all(|character| character.is_alphanumeric() || matches!(character, '_' | '-' | '.'))
    {
        return Err("Username may only contain letters, digits, '_', '-' and '.'.".to_owned());
    }

    let mut parts = email.split('@');
    let local = parts.next().unwrap_or_default();
    let domain = parts.next().unwrap_or_default();
    if local.is_empty() || domain.is_empty() || parts.next().is_some() || !domain.contains('.') {
        return Err("Enter a valid email address.".to_owned());
    }

    if password.chars().count() < 8 {
        return Err("Password must be at least 8 characters long.".to_owned());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registration_rejects_empty_fields() {
        assert!(validate_registration("", "a@b.com", "secret123").is_err());
        assert!(validate_registration("alice", "", "secret123").is_err());
        assert!(validate_registration("alice", "a@b.com", "").is_err());
    }

    #[test]
    fn registration_mirrors_server_rules() {
        assert!(validate_registration("alice", "alice@example.com", "secret123").is_ok());
        assert!(validate_registration("alice!", "alice@example.com", "secret123").is_err());
        assert!(validate_registration("alice", "alice.example.com", "secret123").is_err());
        assert!(validate_registration("alice", "alice@example.com", "short").is_err());
    }

    #[test]
    fn login_rejects_empty_fields() {
        assert!(validate_login("", "secret123").is_err());
        assert!(validate_login("alice", "").is_err());
        assert!(validate_login("alice", "secret123").is_ok());
    }
}
