pub mod sora {
    use std::io::prelude::*;

    use rand::Rng;

    use reqwest::Client;
    use tokio::fs::{self, File};
    use tokio::io::AsyncReadExt;

    use serde::{Serialize, Deserialize};

    use elefren::status_builder::Visibility;
    use elefren::entities::notification::Notification;
    use elefren::entities::notification::NotificationType;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Mstdn {
        pub instance: String,
        pub token: String
    }

    // pub async fn get_words() -> Vec<&'static str> {
    //     let mut file = File::open("words.toml").await.unwrap();
    //     let mut data = String::new();

    //     file.read_to_string(&mut data).await
    //         .expect("failed to read words.toml");
        
    //     let map: HashMap<String, Vec<&str>> = toml::from_str(data.as_str()).unwrap();
    //     let m: Vec<&'static str> = map.get("words").unwrap();
    //     m
    // }

    async fn message_builder(reply_to: String, words: &Vec<&str>) -> String {
        let mut rng = rand::thread_rng();
        
        format!(
            "@{} {}",
            reply_to,
            words.get(rng.gen_range(0..words.len())).unwrap()
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

    pub async fn execute(ref notification: Notification, core: &Mstdn, words: &Vec<&str>, http: Client) -> Result<(), Box<dyn std::error::Error>> {
        match notification.account.bot {
            Some(bot) => {
                if bot == true {
                    return Ok(());
                }
            },
            None => { return Ok(()); }
        }
        if notification.notification_type != NotificationType::Mention {
            return Ok(());
        }

        let s = notification.status.clone().unwrap();

        let reply_to = notification.account.acct.clone();
        let message = message_builder(reply_to, words).await;

        let mut visibility = s.visibility;

        if visibility == Visibility::Public {
            visibility = Visibility::Unlisted;
        }

        let status = elefren::status_builder::StatusBuilder::new()
            .status(message)
            .in_reply_to(s.id)
            .visibility(visibility)
            .build()?;

        http
            .post(format!("{}/api/v1/statuses", core.instance.clone()))
            .bearer_auth(core.token.clone())
            .form(&status)
            .send()
            .await?;

        Ok(())
    }
}