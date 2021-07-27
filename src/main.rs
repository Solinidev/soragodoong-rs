mod soragodoong;
pub use crate::soragodoong::sora;

use elefren::prelude::*;
use elefren::entities::event::Event;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let core = sora::get_core().await;

    let data = Data {
        base: core.instance.into(),
        token: core.token.into(),
        client_id: "".into(),
        client_secret: "".into(),
        redirect: "".into()
    };

    let client = Mastodon::from(data);

    for event in client.streaming_user()? {
        match event {
            Event::Notification(ref notification) => {
                if let Err(w) = sora::execute(notification.to_owned()).await {
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