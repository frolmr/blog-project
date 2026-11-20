use web_sys::Storage;

pub const TOKEN_KEY: &str = "blog_token";

pub fn save_token_to_storage(token: &str) -> Result<(), String> {
    storage()?
        .set_item(TOKEN_KEY, token)
        .map_err(|_| "failed to save the token in localStorage".to_owned())
}

pub fn get_token_from_storage() -> Option<String> {
    storage()
        .ok()?
        .get_item(TOKEN_KEY)
        .ok()
        .flatten()
        .filter(|token| !token.trim().is_empty())
}

pub fn clear_token_from_storage() {
    if let Ok(storage) = storage() {
        let _ = storage.remove_item(TOKEN_KEY);
    }
}

fn storage() -> Result<Storage, String> {
    web_sys::window()
        .ok_or_else(|| "the browser window is unavailable".to_owned())?
        .local_storage()
        .map_err(|_| "localStorage is not accessible".to_owned())?
        .ok_or_else(|| "localStorage is not available".to_owned())
}
