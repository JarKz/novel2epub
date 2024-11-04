use std::time::Duration;

use image::{codecs::png::PngEncoder, GenericImageView, ImageEncoder};
use reqwest::{Client, ClientBuilder, StatusCode};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;

use crate::make_public;

make_public! {
    #[derive(Debug, Deserialize)]
    struct Info {
        cover: Cover,
        name: String,
        rus_name: String,
    }
}

make_public! {
    #[derive(Debug, Deserialize)]
    struct Cover {
        default: String,
        #[allow(unused)]
        thumbnail: String,
    }
}

#[derive(Debug, Deserialize)]
struct ChapterInfo {
    number: String,
    volume: String,
}

make_public! {
    #[derive(Debug, Deserialize)]
    struct Chapter {
        name: String,
        number: String,
        volume: String,
        content: String,
    }
}

pub struct MangalibApi {
    client: Client,
}

impl MangalibApi {
    pub fn new() -> Self {
        Self {
            client: ClientBuilder::new()
                .user_agent("Mozilla/5.0")
                .cookie_store(true)
                .build()
                .expect("successful reqwest client build"),
        }
    }

    pub async fn get_info(&self, name: &str) -> anyhow::Result<Info> {
        let url = get_url(name);
        get_data(&self.client.get(url).send().await?.text().await?)
    }

    pub async fn get_chapters(&self, name: &str) -> anyhow::Result<Vec<Chapter>> {
        let url_template = get_url(name);
        let url = format!("{url_template}/chapters");
        let chapter_infos: Vec<ChapterInfo> =
            get_data(&self.client.get(url).send().await?.text().await?)?;

        let fetched_chapters = chapter_infos
            .iter()
            .map(|chapter_info| async {
                let request_url = format!(
                    "{url_template}/chapter?number={}&volume={}",
                    chapter_info.number, chapter_info.volume
                );

                let make_request = || async {
                    self.client
                        .get(&request_url)
                        .send()
                        .await
                        .expect("successful request")
                };
                let mut response = make_request().await;
                while response.status() != StatusCode::OK {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    response = make_request().await;
                }
                response
            })
            .map(|response| async {
                let json = response
                    .await
                    .text()
                    .await
                    .expect("get json as plain string");

                get_data::<Chapter>(&json)
            });

        let mut chapters = Vec::with_capacity(chapter_infos.len());
        for fetched_chapter in fetched_chapters {
            chapters.push(fetched_chapter.await?);
        }

        Ok(chapters)
    }

    pub async fn get_image(&self, url: String) -> anyhow::Result<Vec<u8>> {
        let bytes = self.client.get(url).send().await?.bytes().await?;
        let image = image::load_from_memory(&bytes)?;

        let mut data = vec![];
        let encoder = PngEncoder::new(&mut data);
        encoder.write_image(
            image.as_bytes(),
            image.dimensions().0,
            image.dimensions().1,
            image.color().into(),
        )?;

        Ok(data)
    }
}

fn get_url(name: &str) -> String {
    format!("https://api.mangalib.me/api/manga/{name}")
}

fn get_data<T: DeserializeOwned>(json: &str) -> anyhow::Result<T> {
    Ok(serde_json::from_value(
        serde_json::from_str::<Value>(json)?
            .get("data")
            .unwrap()
            .clone(),
    )?)
}
