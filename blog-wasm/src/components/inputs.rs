//! Reusable input callbacks for controlled form fields.
//!
//! The helpers copy the current DOM value into a Yew state handle so components
//! stay controlled and can reset fields after a submit.

use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

/// Returns an input callback that updates `state` with a text input's value.
pub fn on_input(state: UseStateHandle<String>) -> Callback<InputEvent> {
    Callback::from(move |event: InputEvent| {
        state.set(event.target_unchecked_into::<HtmlInputElement>().value());
    })
}

/// Returns an input callback that updates `state` with a textarea's value.
pub fn on_textarea_input(state: UseStateHandle<String>) -> Callback<InputEvent> {
    Callback::from(move |event: InputEvent| {
        state.set(event.target_unchecked_into::<HtmlTextAreaElement>().value());
    })
}
