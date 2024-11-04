use clap::Parser;
use clap_derive::Parser;
use converter::generate_epub;
use mangalib_api::MangalibApi;

mod converter;
mod declarative_macros;
mod mangalib_api;

/// The application that can convert any possible novel from RanobeLib into EPUB document for usage
/// in e-ink readers.
#[derive(Parser)]
struct Command {
    /// The URL of novel's main page in RanobeLib
    #[clap(short, long)]
    url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let command = Command::parse();
    let url_before_query_params = command.url.split_once('?').unwrap_or((&command.url, "")).0;
    let novel_name = url_before_query_params
        .split('/')
        .rev()
        .next()
        .expect("expected an name of novel");

    let api = MangalibApi::new();
    let novel_info = api.get_info(novel_name).await?;
    let novel_chapters = api.get_chapters(novel_name);
    let cover = api.get_image(novel_info.cover.default.clone());

    generate_epub(novel_info, cover.await?, novel_chapters.await?)
        .expect("generate a epub document");

    Ok(())
}
