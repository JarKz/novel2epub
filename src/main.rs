use converter::generate_epub;
use mangalib_api::MangalibApi;

mod converter;
mod declarative_macros;
mod mangalib_api;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let novel_name = "11990--jubuldo-absnen-hoegu";

    let api = MangalibApi::new();
    let novel_info = api.get_info(novel_name).await?;
    let novel_chapters = api.get_chapters(novel_name);
    let cover = api.get_image(novel_info.cover.default.clone());

    generate_epub(novel_info, cover.await?, novel_chapters.await?)
        .expect("generate a epub document");

    Ok(())
}
