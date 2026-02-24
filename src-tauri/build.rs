fn main() {
    // Load .env
    dotenv::dotenv().ok();
    let salt = std::env::var("SALT")
        .unwrap_or_else(|_| {
            eprintln!("SALT env variable not set.");
            "SALT".to_string()
        });
    println!("cargo:rustc-env=SALT={}", salt);
    
    tauri_build::build()
}
