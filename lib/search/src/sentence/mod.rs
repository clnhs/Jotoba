mod order;
pub mod result;

use std::time::Instant;

use super::query::Query;
use crate::query::QueryLang;
use error::Error;
use resources::parse::jmdict::languages::Language;

/// Searches for sentences
pub async fn search(query: &Query) -> Result<(Vec<result::Item>, usize), Error> {
    use crate::engine::sentences::{foreign, japanese};

    let start = Instant::now();

    let lang = query.settings.user_lang;
    let show_english = query.settings.show_english;

    let res = if query.language == QueryLang::Japanese {
        japanese::Find::new(&query.query, 1000, 0)
            .with_language_filter(query.settings.user_lang)
            .find()
            .await?
    } else {
        let mut res = foreign::Find::new(&query.query, query.settings.user_lang, 1000, 0)
            .find_engish(show_english)
            .find()
            .await?;

        if res.len() < 20 && show_english {
            res.extend(foreign::Find::new(&query.query, Language::English, 1000, 0)
                .find()
                .await?);
        }

        res
    };

    let sentence_storage = resources::get().sentences();

    let sentences = res
        .retrieve_ordered(|i| sentence_storage.by_id(i as u32))
        .collect::<Vec<_>>();

    let len = sentences.len();

    let sentences = sentences
        .into_iter()
        .filter_map(|i| result::Sentence::from_m_sentence(i.clone(), lang, show_english))
        .map(|i| result::Item { sentence: i })
        .skip(query.page_offset)
        .take(10)
        .collect::<Vec<_>>();

    println!("Sentence search took: {:?}", start.elapsed());

    Ok((sentences, len))
}
