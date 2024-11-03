use std::time::Duration;

use epub_builder::{EpubBuilder, ZipLibrary};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct NovelData {
    data: Novel,
}

#[derive(Debug, Deserialize)]
struct Novel {
    cover: Cover,
    name: String,
    rus_name: String,
}

#[derive(Debug, Deserialize)]
struct Cover {
    default: String,
    thumbnail: String,
}

#[derive(Debug, Deserialize)]
struct NovelChapters {
    data: Vec<NovelChapterInfo>,
}

#[derive(Debug, Deserialize)]
struct NovelChapterInfo {
    name: String,
    number: String,
    volume: String,
}

#[derive(Debug, Deserialize)]
struct NovelChapter {
    name: String,
    number: String,
    volume: String,
    content: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .user_agent("Mozilla/5.0")
        .build()?;

    let template = "https://api.mangalib.me/api/manga";
    let novel_name = "11990--jubuldo-absnen-hoegu";

    let novel_info = client
        .get(format!("{template}/{novel_name}"))
        .header("Accept", "*/*")
        .send()
        .await?;
    let text = novel_info.text().await?;
    let _novel_data: NovelData = serde_json::from_str(&text)?;

    let novel_chapters = client
        .get(format!("{template}/{novel_name}/chapters"))
        .header("Accept", "*/*")
        .send()
        .await?;

    let text = novel_chapters.text().await?;
    let novel_chapters: NovelChapters = serde_json::from_str(&text)?;

    let chapters = novel_chapters
        .data
        .iter()
        .map(|novel_chapter_info| async {
            let url = format!(
                "{template}/{novel_name}/chapter?number={}&volume={}",
                novel_chapter_info.number, novel_chapter_info.volume
            );

            let mut response = client.get(&url).send().await.unwrap();
            while StatusCode::OK != response.status() {
                tokio::time::sleep(Duration::from_secs(1)).await;
                response = client.get(&url).send().await.unwrap();
            }

            response
        })
        .map(|response| async {
            //TODO: change unwrap to recoverable request to get properly a chapter
            let text = response.await.text().await.unwrap();
            let novel_chapter: NovelChapter = serde_json::from_value(
                serde_json::from_str::<Value>(&text)
                    .unwrap()
                    .get("data")
                    .unwrap()
                    .clone(),
            )
            .unwrap();

            novel_chapter
        });

    let mut novel_chapters = Vec::with_capacity(novel_chapters.data.len());

    for chapter in chapters {
        let chapter = chapter.await;
        println!("Chapter {} added!", chapter.name);
        novel_chapters.push(chapter);
    }

    Ok(())
}

//TODO: create a epub document
fn create_epub(
    novel_info: NovelData,
    novel_chapters: Vec<NovelChapter>,
) -> epub_builder::Result<Vec<u8>> {
    let mut output = vec![];
    EpubBuilder::new(ZipLibrary::new()?)?
        .metadata("title", novel_info.data.rus_name)?
        .epub_version(epub_builder::EpubVersion::V30)
        .add_cover_image(
            "cover_image.png",
            novel_info.data.cover.default.as_bytes(),
            "image/png",
        )?
        .generate(&mut output)?;

    Ok(output)
}
