#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let res = rfd::FileDialog::new()
        .add_filter("text", &["txt", "rs"])
        .add_filter("rust", &["rs", "toml"])
        .set_directory(&"/")
        .pick_files();

    println!("The user choose: {:#?}", res);
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // On wasm only async dialogs are posible
}
