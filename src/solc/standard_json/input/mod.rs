//!
//! The `solc --standard-json` input representation.
//!

pub mod language;
pub mod settings;
pub mod source;

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use self::language::Language;
use self::settings::Settings;
use self::source::Source;

///
/// The `solc --standard-json` input representation.
///
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    /// The input language.
    pub language: Language,
    /// The input source code files hashmap.
    pub sources: HashMap<String, Source>,
    /// The compiler settings.
    pub settings: Settings,
}

impl Input {
    ///
    /// A shortcut constructor.
    ///
    pub fn try_from_paths(
        language: Language,
        paths: &[PathBuf],
        library_map: Vec<String>,
        output_selection: serde_json::Value,
        optimize: bool,
    ) -> anyhow::Result<Self> {
        let mut sources = HashMap::with_capacity(paths.len());
        for path in paths.iter() {
            let source = Source::try_from(path.as_path())?;
            sources.insert(path.to_string_lossy().to_string(), source);
        }

        let libraries = Settings::parse_libraries(library_map)?;

        Ok(Self {
            language,
            sources,
            settings: Settings::new(libraries, output_selection, optimize),
        })
    }

    ///
    /// A shortcut constructor.
    ///
    /// Only for the integration test purposes.
    ///
    pub fn try_from_sources(
        sources: HashMap<String, String>,
        libraries: HashMap<String, HashMap<String, String>>,
        output_selection: serde_json::Value,
        optimize: bool,
    ) -> anyhow::Result<Self> {
        let sources = sources
            .into_iter()
            .map(|(path, content)| (path, Source::from(content)))
            .collect();

        Ok(Self {
            language: Language::Solidity,
            sources,
            settings: Settings::new(libraries, output_selection, optimize),
        })
    }
}
