mod assets {
    #![allow(unused_imports, unused_variables, unused_results, clippy::redundant_pub_crate)]
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

use std::{collections::HashMap, sync::LazyLock};

pub static WEB_UI_ASSETS: LazyLock<HashMap<&'static str, static_files::Resource>> =
    LazyLock::new(assets::generate);

#[cfg(test)]
mod tests {
    use crate::WEB_UI_ASSETS;

    #[test]
    fn test_assets() {
        assert!(!WEB_UI_ASSETS.is_empty());

        let mut entries = WEB_UI_ASSETS.iter().collect::<Vec<_>>();
        entries.sort_unstable_by_key(|entry| entry.0);
        for (file_name, resource) in entries {
            println!("{file_name}\t{}", resource.data.len());
        }
    }
}
