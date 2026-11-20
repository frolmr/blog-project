use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

pub fn on_input(state: UseStateHandle<String>) -> Callback<InputEvent> {
    Callback::from(move |event: InputEvent| {
        state.set(event.target_unchecked_into::<HtmlInputElement>().value());
    })
}

pub fn on_textarea_input(state: UseStateHandle<String>) -> Callback<InputEvent> {
    Callback::from(move |event: InputEvent| {
        state.set(event.target_unchecked_into::<HtmlTextAreaElement>().value());
    })
}
