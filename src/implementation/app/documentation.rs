use std::path::PathBuf;

use crate::app::AppLanguage;

pub(super) fn documentation_markdown_path(language: AppLanguage) -> Option<PathBuf> {
    let file_name = match language {
        AppLanguage::English => "Documentation.md",
        AppLanguage::SimplifiedChinese => "Documentation.zh-CN.md",
    };
    let candidates = [
        std::env::current_dir()
            .ok()
            .map(|cwd| cwd.join("assets").join("help").join(file_name)),
        std::env::current_exe().ok().and_then(|exe| {
            exe.parent()
                .map(|dir| dir.join("../Resources/help").join(file_name))
        }),
        std::env::current_exe().ok().and_then(|exe| {
            exe.parent()
                .and_then(|dir| dir.parent())
                .map(|contents| contents.join("Resources/help").join(file_name))
        }),
    ];

    candidates
        .into_iter()
        .flatten()
        .find(|path| path.exists())
        .or_else(|| {
            (language != AppLanguage::English)
                .then(|| documentation_markdown_path(AppLanguage::English))
                .flatten()
        })
}
