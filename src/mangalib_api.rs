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

            match fetched_chapter.await {
                Ok(chapter) => {
                    println!("Downloaded a chapter: {}", chapter.name);
                    chapters.push(chapter);
                }
                Err(e) => {
                    println!("Error while downloading a chapter: {:?}", e);
                    // Skip the chapter if it failed to download
                    continue;
                }
            }
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
    // Parse the full JSON string first
    let mut full_json: Value = serde_json::from_str(json)?;
    
    // Extract the "data" field
    let data_value = full_json.get_mut("data")
        .ok_or_else(|| anyhow::anyhow!("Missing 'data' field in JSON"))?;
    
    // Check if this is a chapter with structured content and convert to string if needed
    if let Some(obj) = data_value.get("content").and_then(|c| c.as_object()) {
        // It's a structured content - convert to simplified string
        if data_value.is_object() {
            let content_obj = data_value["content"].clone();
            // Replace the structured content with a simple string representation
            data_value["content"] = Value::String(content_obj.to_string());
        }
    }
    
    // Deserialize the modified data field into the target type
    match serde_json::from_value(data_value.clone()) {
        Ok(data) => Ok(data),
        Err(e) => {
            // Log error details for debugging
            println!("JSON parsing error: {:?}", e);
            println!("Problem JSON data: {}", 
                &data_value.to_string()[..std::cmp::min(100, data_value.to_string().len())]);
            Err(anyhow::anyhow!("Failed to parse JSON data: {}", e))
        }
    }
}