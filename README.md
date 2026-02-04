# About

The Rust application which can parse particular novel by URL from ranobelib and convert into epub.

> [!WARNING]
> RanobeLib updated their API, and it's no longer available for straightforward use, breaking the application.
> The Author (me) is not interested to fix this, because it increases the complexity of use.
> If you're interested to make the application independent of RanobeLib and use other API, open an issue with proposal.

## Usage

Visit any novel's main page in RanobeLib. For example, `https://ranobelib.me/ru/book/26690--omniscient-readers-viewpoint-novel?section=info&ui=3716286`
and copy the URL from bar in browser. Pass the url into application:

```bash
cargo run -- --url {insert_url_here}
```

Wait some time and the EPUB document is ready!

## The simplicity of EPUB

The main goal that make readable document for any e-ink readers like Amazon Kindle Paperwhite/Oasis/Scribe.
So I use only cover, chapters and table of contents. I didn't make any attention into novel author, simple
differences and etc.

## Detail information

Uses the mangalib API to get desired result because scraping won't work because of JS.
The RanobeLib have an updated frontend where content dynamically loades by javascript, that's why I use the API.
I can't tell where to get any information about API, so I did a research of site and tracked the requests.

I can tell only part of API:

 - information about manga/novel;
 - list of chapters;
 - chapter with content;

I'll follow each step by step. Firstly, for all requests uses a simple HTTP(S) template: `https://api.mangalib.me/api/manga`. 
For all methods below described, need to only add a method at the end of template with query parameters when needed.

Let's begin with 'information about manga/novel'. Every manga/novel have a name which is different to real name because there is added
some symbols that makes harder to find the exact manga/novel. So need to visit any manga/novel page at mangalib/ranobelib and copypaste
the name which is usually placed at the end, before query params. To fetch any information about manga/novel in JSON format, need to use
template above and put the name of manga/novel: `https://api.mangalib.me/api/manga/{name}`. Here possible some query parameters, but I
didn't make a research deeper because for me it's useless.

To get list of chapters need use the template: `https://api.mangalib.me/api/manga/{name}/chapters`.

To get detail about chapter, use the template: `https://api.mangalib.me/api/manga/{name}/chapter?volume={volume}&number={number}`, where
`volume` and `number` can be taken in previous request.

I didn't go deeper into detail about each api because each of you can test, using any browser like firefox (fuck you, google chrome).
And that's all API which I use to make epub document from information which fetched by API above.

## Why the downloading is so long?

It's because the mangalib API have a restrictment by request count. Around 100 requests per some seconds.
So be patient and wait before the code resolves and downloads the remaining chapters.
