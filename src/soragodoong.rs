pub mod sora {
    use std::{collections::HashMap, io::prelude::*};

    use rand::Rng;

    use tokio::fs::{self, File};
    use tokio::io::AsyncReadExt;

    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Mstdn {
        pub instance: String,
        pub token: String
    }

    async fn message_builder(reply_to: String) -> String {
        let mut file = File::open("words.toml").await.unwrap();
        let mut data = String::new();

        file.read_to_string(&mut data).await
            .expect("failed to read words.toml");

        let words_map: HashMap<String, Vec<&str>> = toml::from_str(data.as_str()).unwrap();
        let words_list = words_map.get("words").unwrap();

        let mut rng = rand::thread_rng();
        
        format!(
            "@{} {}",
            reply_to,
            words_list.get(rng.gen_range(0..words_list.len())).unwrap()
        )
    }

    pub async fn get_core() -> Mstdn {
        if std::path::Path::new("core.toml").is_file() {
            let mut file = File::open("core.toml").await.unwrap();
            let mut data = String::new();
            file.read_to_string(&mut data).await
                .expect("failed");
            let m: Mstdn = toml::from_str(data.as_str()).unwrap();

            return m;
        } else {
            let mut instance = String::new();
            let mut token = String::new();

            print!("Input your istance address : ");
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut instance)
                .expect("failed");
            instance = String::from(instance.trim());

            print!("Input your access token : ");
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut token)
                .expect("failed");
            token = String::from(token.trim());
            
            let core = Mstdn {
                instance: instance,
                token: token,
            };

            let toml_string = toml::to_string(&core).unwrap();
            fs::write("core.toml", toml_string).await
                .expect("failed to write");

            return core;
        }
    }

    pub async fn execute(ref notification: elefren::entities::notification::Notification) -> Result<(), Box<dyn std::error::Error>> {
        match notification.account.bot {
            Some(bot) => {
                if bot == true {
                    return Ok(());
                }
            },
            None => { return Ok(()); }
        }
        if notification.notification_type != elefren::entities::notification::NotificationType::Mention {
            return Ok(());
        }

        let core = get_core().await;

        let s = notification.status.clone().unwrap();

        let reply_to = notification.account.acct.clone();
        let message = message_builder(reply_to).await;

        let mut visibility = s.visibility;

        if visibility == elefren::status_builder::Visibility::Public {
            visibility = elefren::status_builder::Visibility::Unlisted;
        }

        let status = elefren::status_builder::StatusBuilder::new()
            .status(message)
            .in_reply_to(s.id)
            .visibility(visibility)
            .build()?;

        reqwest::Client::new()
            .post(format!("{}/api/v1/statuses", core.instance))
            .bearer_auth(core.token)
            .form(&status)
            .send()
            .await?;

        Ok(())
    }
}