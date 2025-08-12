mod embedding;
mod ino_loader;

use ino_loader::loader::load;

use dotenv::dotenv;
use std::env;

use embedding::vectorize::get_embedding;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let res = load("assets/export.xlsx", "ino").ok();

    let yandex_token = env::var("YANDEX_SECRET").ok().unwrap();
    let yandex_model = env::var("YANDEX_MODEL").ok().unwrap();
    let yandex_url = env::var("YANDEX_URL").ok().unwrap();

    for item in res.unwrap().iter() {
        let emb_model = get_embedding(&item.name, &yandex_model, &yandex_token, &yandex_url).await;

        match emb_model {
            Err(e) => eprintln!("{e}"),
            Ok(emb) => {
                if let Some(t) = emb.embedding {
                    println!("{} - {}", item.name, t.len());
                } else if let Some(eq) = emb.error {
                    println!("{} - {eq}", item.name);
                }
            }
        }
    }
}
