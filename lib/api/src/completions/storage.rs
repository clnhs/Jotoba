use std::{collections::HashMap, error::Error, fs::File, io::BufReader, path::Path};

use autocompletion::index::{basic::BasicIndex, japanese::JapaneseIndex};
use config::Config;
use log::info;
use once_cell::sync::OnceCell;
use search::suggestions::{store_item, TextSearch};
use serde::{Deserialize, Deserializer};
use types::jotoba::languages::Language;

use super::WordPair;

// Words
pub static JP_WORD_INDEX: OnceCell<JapaneseIndex> = OnceCell::new();
pub static WORD_INDEX: OnceCell<HashMap<Language, BasicIndex>> = OnceCell::new();

/// Kanji meanings
pub(crate) static K_MEANING_SUGGESTIONS: OnceCell<JapaneseIndex> = OnceCell::new();

/// In-memory storage for native name suggestions
pub(crate) static NAME_NATIVE: OnceCell<TextSearch<Vec<NameNative>>> = OnceCell::new();

/// In-memory storage for name transcriptions suggestions
pub(crate) static NAME_TRANSCRIPTIONS: OnceCell<TextSearch<Vec<NameTranscription>>> =
    OnceCell::new();

/// Load all available suggestions
pub fn load_suggestions(config: &Config) -> Result<(), Box<dyn Error>> {
    rayon::scope(|s| {
        s.spawn(|_| {
            if let Err(err) = load_words(config) {
                eprintln!("Error loading word suggestions {}", err);
            }
        });
        s.spawn(|_| {
            if let Err(err) = load_meanings(config) {
                eprintln!("Error loading meaning suggestions {}", err);
            }
        });
        s.spawn(|_| {
            if let Err(err) = load_name_transcriptions(config) {
                eprintln!("Error loading name suggestions {}", err);
            }
        });
        s.spawn(|_| {
            if let Err(err) = load_native_names(config) {
                eprintln!("Error loading name suggestions {}", err);
            }
        });
    });
    Ok(())
}

// Load word suggestion index
fn load_words(config: &Config) -> Result<(), Box<dyn Error>> {
    let mut index_map: HashMap<Language, BasicIndex> = HashMap::with_capacity(9);
    for language in Language::iter() {
        let path = Path::new(config.get_suggestion_sources()).join(format!("new_word_{language}"));
        if !path.exists() {
            log::warn!("Running without {language} suggestions");
            continue;
        }
        if language != Language::Japanese {
            let index: BasicIndex = bincode::deserialize_from(BufReader::new(File::open(path)?))?;
            index_map.insert(language, index);
        } else {
            let index: JapaneseIndex =
                bincode::deserialize_from(BufReader::new(File::open(path)?))?;
            JP_WORD_INDEX.set(index).ok().expect("JP Index alredy set");
        }
    }

    WORD_INDEX
        .set(index_map)
        .ok()
        .expect("WORD_INDEX already set!");

    Ok(())
}

/// Load kanji meaning suggestion file into memory
fn load_meanings(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let file = Path::new(config.get_suggestion_sources()).join("kanji_meanings");
    if !file.exists() {
        info!("Kanji-meaning suggestion file does not exists");
        return Ok(());
    }

    let index: JapaneseIndex = bincode::deserialize_from(BufReader::new(File::open(file)?))?;
    K_MEANING_SUGGESTIONS
        .set(index)
        .ok()
        .expect("won't happen lol");

    Ok(())
}

/// A suggestion Item for transcribed a name
#[derive(Deserialize)]
pub struct NameTranscription {
    pub name: String,
    #[serde(deserialize_with = "eudex_deser")]
    pub hash: eudex::Hash,
}

/// A suggestion Item for a name item in japanese
#[derive(Deserialize)]
pub struct NameNative {
    pub name: String,
}

impl store_item::Item for NameTranscription {
    #[inline]
    fn get_text(&self) -> &str {
        &self.name
    }

    #[inline]
    fn get_hash(&self) -> eudex::Hash {
        self.hash
    }
}

impl Into<WordPair> for &NameTranscription {
    #[inline]
    fn into(self) -> WordPair {
        WordPair {
            primary: self.name.clone(),
            ..Default::default()
        }
    }
}

impl store_item::Item for NameNative {
    #[inline]
    fn get_text(&self) -> &str {
        &self.name
    }
}

impl Into<WordPair> for &NameNative {
    #[inline]
    fn into(self) -> WordPair {
        WordPair {
            primary: self.name.clone(),
            ..Default::default()
        }
    }
}

/// Load native name suggestions
fn load_native_names(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let file = Path::new(config.get_suggestion_sources()).join("names_native");
    if !file.exists() {
        info!("Native name suggestion file does not exists");
        return Ok(());
    }

    let items: Vec<NameNative> = bincode::deserialize_from(BufReader::new(File::open(file)?))?;

    NAME_NATIVE.set(TextSearch::new(items)).ok();

    info!("Loaded native name suggestion file");

    Ok(())
}

/// Load name transcription suggestions
fn load_name_transcriptions(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let file = Path::new(config.get_suggestion_sources()).join("names_trans");
    if !file.exists() {
        info!("Name transcription suggestion file does not exists");
        return Ok(());
    }

    let items: Vec<NameTranscription> =
        bincode::deserialize_from(BufReader::new(File::open(file)?))?;

    NAME_TRANSCRIPTIONS.set(TextSearch::new(items)).ok();

    info!("Loaded name transcriptions suggestion file");

    Ok(())
}

#[inline]
fn eudex_deser<'de, D>(deserializer: D) -> Result<eudex::Hash, D::Error>
where
    D: Deserializer<'de>,
{
    let s: u64 = Deserialize::deserialize(deserializer)?;
    Ok(eudex::Hash::from(s))
}
