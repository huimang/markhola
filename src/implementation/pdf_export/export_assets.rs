use std::fs;
use std::path::Path;

use percent_encoding::percent_decode_str;
use pulldown_cmark::{Event, Options, Parser, Tag};
use url::Url;

use crate::document::ActiveDocument;
use crate::render_assets;

pub(super) fn local_image_data_url(document: &ActiveDocument, destination: &str) -> Option<String> {
    let destination_path = local_destination_path(destination)?;
    let document_root = document.file_path().parent()?.canonicalize().ok()?;
    let base_url = Url::from_directory_path(&document_root).ok()?;
    let asset_url = base_url.join(destination_path).ok()?;
    if asset_url.scheme() != "file" || asset_url.query().is_some() || asset_url.fragment().is_some()
    {
        return None;
    }

    let asset_path = asset_url.to_file_path().ok()?.canonicalize().ok()?;
    if !asset_path.starts_with(&document_root) || !asset_path.is_file() {
        return None;
    }

    let mime = supported_image_mime(&asset_path)?;
    let bytes = fs::read(asset_path).ok()?;
    Some(format!(
        "data:{mime};base64,{}",
        render_assets::encode_base64(&bytes)
    ))
}

pub(super) fn validate_local_images(document: &ActiveDocument) -> Result<(), String> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    for event in Parser::new_ext(document.markdown(), options) {
        let Event::Start(Tag::Image { dest_url, .. }) = event else {
            continue;
        };
        let destination = dest_url.as_ref();
        if destination.starts_with('#')
            || destination.contains('?')
            || Url::parse(destination)
                .ok()
                .is_some_and(|url| matches!(url.scheme(), "http" | "https" | "data"))
        {
            continue;
        }
        if local_image_data_url(document, destination).is_none() {
            return Err(format!(
                "missing_local_asset: Local image is unavailable: {destination}"
            ));
        }
    }
    Ok(())
}

fn local_destination_path(destination: &str) -> Option<&str> {
    if destination.is_empty() || !has_valid_percent_encoding(destination) {
        return None;
    }

    if Url::parse(destination).is_ok() {
        return None;
    }

    let relative_url = Url::parse("file:///").ok()?.join(destination).ok()?;
    if relative_url.query().is_some() || relative_url.fragment().is_some() {
        return None;
    }

    let decoded = percent_decode_str(destination).decode_utf8().ok()?;
    if Path::new(decoded.as_ref()).is_absolute() {
        return None;
    }

    Some(destination)
}

fn has_valid_percent_encoding(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'%' {
            index += 1;
            continue;
        }
        if index + 2 >= bytes.len()
            || !bytes[index + 1].is_ascii_hexdigit()
            || !bytes[index + 2].is_ascii_hexdigit()
        {
            return false;
        }
        index += 3;
    }
    true
}

fn supported_image_mime(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "svg" => Some("image/svg+xml"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use crate::document::ActiveDocument;
    use crate::file_io;
    use crate::markdown;

    use super::local_image_data_url;

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(1);

    fn test_document(markdown: &str) -> (PathBuf, ActiveDocument) {
        let root = std::env::temp_dir().join(format!(
            "markhola-export-assets-{}-{}",
            std::process::id(),
            NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("assets")).expect("test directory should be created");
        let document_path = root.join("document.md");
        fs::write(&document_path, markdown).expect("test document should be written");
        let base_url =
            file_io::directory_base_url(&document_path).expect("base URL should resolve");
        (
            root,
            ActiveDocument::open_with_id(1, document_path, markdown.to_string(), base_url),
        )
    }

    fn remove_test_directory(root: &Path) {
        fs::remove_dir_all(root).expect("test directory should be removed");
    }

    #[test]
    fn inlines_supported_local_images_with_mime_types_and_encoded_spaces() {
        let (root, document) = test_document("# Export assets");
        fs::write(root.join("assets/diagram.svg"), b"<svg></svg>").expect("SVG should be written");
        fs::write(root.join("assets/sample image.png"), b"png").expect("PNG should be written");
        fs::write(root.join("assets/photo.jpg"), b"jpg").expect("JPG should be written");
        fs::write(root.join("assets/photo.jpeg"), b"jpeg").expect("JPEG should be written");
        fs::write(root.join("assets/animation.gif"), b"gif").expect("GIF should be written");
        fs::write(root.join("assets/sample.webp"), b"webp").expect("WebP should be written");

        let svg =
            local_image_data_url(&document, "./assets/diagram.svg").expect("SVG should be inlined");
        let png = local_image_data_url(&document, "./assets/sample%20image.png")
            .expect("encoded PNG path should be inlined");
        let png_with_literal_space = local_image_data_url(&document, "./assets/sample image.png")
            .expect("literal-space PNG path should be inlined");

        assert!(svg.starts_with("data:image/svg+xml;base64,"));
        assert!(png.starts_with("data:image/png;base64,"));
        assert_eq!(png_with_literal_space, png);
        assert!(
            local_image_data_url(&document, "./assets/photo.jpg")
                .expect("JPG should be inlined")
                .starts_with("data:image/jpeg;base64,")
        );
        assert!(
            local_image_data_url(&document, "./assets/photo.jpeg")
                .expect("JPEG should be inlined")
                .starts_with("data:image/jpeg;base64,")
        );
        assert!(
            local_image_data_url(&document, "./assets/animation.gif")
                .expect("GIF should be inlined")
                .starts_with("data:image/gif;base64,")
        );
        assert!(
            local_image_data_url(&document, "./assets/sample.webp")
                .expect("WebP should be inlined")
                .starts_with("data:image/webp;base64,")
        );
        remove_test_directory(&root);
    }

    #[test]
    fn rejects_missing_unsupported_absolute_and_escaping_images() {
        let (root, document) = test_document("# Export assets");
        let outside = root.parent().expect("test root should have a parent").join(
            root.file_name()
                .expect("test root should have a name")
                .to_string_lossy()
                .to_string()
                + "-outside.svg",
        );
        fs::write(&outside, b"<svg></svg>").expect("outside SVG should be written");
        symlink(&outside, root.join("assets/symlink.svg")).expect("symlink should be created");
        fs::write(root.join("assets/plain.txt"), b"not an image")
            .expect("unsupported file should be written");

        assert_eq!(
            local_image_data_url(&document, "./assets/missing.svg"),
            None
        );
        assert_eq!(local_image_data_url(&document, "./assets/plain.txt"), None);
        assert_eq!(
            local_image_data_url(&document, outside.to_str().unwrap()),
            None
        );
        assert_eq!(
            local_image_data_url(
                &document,
                &format!(
                    "../{}",
                    outside
                        .file_name()
                        .expect("outside SVG should have a name")
                        .to_string_lossy()
                )
            ),
            None
        );
        assert_eq!(
            local_image_data_url(&document, "./assets/symlink.svg"),
            None
        );
        assert_eq!(
            local_image_data_url(&document, "./assets/diagram%ZZ.svg"),
            None
        );

        fs::remove_file(outside).expect("outside SVG should be removed");
        remove_test_directory(&root);
    }

    #[test]
    fn leaves_remote_data_query_and_fragment_destinations_unchanged() {
        let markdown = concat!(
            "![remote](https://example.com/image.png)\n\n",
            "![data](data:image/png;base64,cG5n)\n\n",
            "![query](./assets/image.png?version=1)\n\n",
            "![fragment](./assets/image.svg#icon)\n\n",
            "![missing](./assets/missing.svg)\n"
        );
        let (root, document) = test_document(markdown);
        let html = markdown::render_html_with_image_resolver(markdown, |destination| {
            local_image_data_url(&document, destination)
        });

        assert!(html.contains("src=\"https://example.com/image.png\""));
        assert!(html.contains("src=\"data:image/png;base64,cG5n\""));
        assert!(html.contains("src=\"./assets/image.png?version=1\""));
        assert!(html.contains("src=\"./assets/image.svg#icon\""));
        assert!(html.contains("src=\"./assets/missing.svg\""));
        remove_test_directory(&root);
    }
}
