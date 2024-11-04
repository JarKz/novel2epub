use std::fs::File;

use epub_builder::{EpubBuilder, EpubContent, ZipLibrary};

use crate::mangalib_api::{Chapter, Info};

pub fn generate_epub(
    novel_info: Info,
    cover: Vec<u8>,
    novel_chapters: Vec<Chapter>,
) -> epub_builder::Result<()> {
    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

    builder
        .metadata("title", novel_info.rus_name)?
        .epub_version(epub_builder::EpubVersion::V30)
        .add_cover_image("cover_image.png", &*cover, "image/png")?;

    for chapter in &novel_chapters {
        builder.add_content(
            EpubContent::new(
                format!("volume_{}_number_{}.xhtml", chapter.volume, chapter.number),
                make_xhtml_content(chapter).as_bytes(),
            )
            .title(chapter.name.clone())
            .reftype(epub_builder::ReferenceType::Text),
        )?;
    }

    let mut file = File::create(format!("{}.epub", novel_info.name))?;
    builder.inline_toc().generate(&mut file)?;

    Ok(())
}

fn make_xhtml_content(chapter: &Chapter) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" ?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en">
  <head>
    <meta http-equiv="Content-Type" content="application/xhtml+xml; charset=utf-8" />
    <title>{title}</title>
    <link rel="stylesheet" href="css/main.css" type="text/css" />
  </head>
  <body>
  <h1>{title}</h1>
  {content}
  </body>
</html>"#,
        title = chapter.name,
        content = chapter.content.replace("<p>\u{a0}</p>", ""),
    )
}
