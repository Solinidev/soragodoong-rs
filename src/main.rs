mod soragodoong;
pub use crate::soragodoong::sora;

use elefren::prelude::*;
use elefren::entities::event::Event;

use tokio::fs::File;
use tokio::io::AsyncReadExt;

use std::error::Error;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut file = File::open("words.toml").await.unwrap();
    let mut data = String::new();

    file.read_to_string(&mut data).await
        .expect("failed to read words.toml");

    let words_map: HashMap<String, Vec<&str>> = toml::from_str(data.as_str()).unwrap();
    let words_list = words_map.get("words").unwrap().to_owned();

    let core = sora::get_core().await;

    let data = Data {
        base: core.instance.clone().into(),
        token: core.token.clone().into(),
        client_id: "".into(),
        client_secret: "".into(),
        redirect: "".into()
    };

    let client = Mastodon::from(data);
    let http_client = reqwest::Client::new();

    for event in client.streaming_user()? {
        match event {
            Event::Notification(ref notification) => {
                if let Err(w) = sora::execute(notification.to_owned(), &core, &words_list, &http_client).await {
                    println!("Error : {:?}", w);
                }
            },
            Event::Update(ref _status) => (),
            Event::Delete(ref _id) => (),
            Event::FiltersChanged => (),
        }
    }

    Ok(())
}