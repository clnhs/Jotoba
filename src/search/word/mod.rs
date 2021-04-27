mod kanji;
mod order;
pub mod result;
mod wordsearch;

use order::{GlossWordOrder, NativeWordOrder};
use result::{Item, Word};
pub use wordsearch::WordSearch;

use async_std::sync::Mutex;
use itertools::Itertools;
use once_cell::sync::Lazy;
use std::time::SystemTime;

use crate::{
    cache::SharedCache,
    error::Error,
    japanese::JapaneseExt,
    models::kanji::Kanji as DbKanji,
    search::{
        query::{Query, QueryLang},
        SearchMode,
    },
    utils::real_string_len,
    DbPool,
};

use self::result::WordResult;

use super::query::Form;

/// An in memory Cache for word search results
static SEARCH_CACHE: Lazy<Mutex<SharedCache<u64, WordResult>>> =
    Lazy::new(|| Mutex::new(SharedCache::with_capacity(1000)));

pub(self) struct Search<'a> {
    db: &'a DbPool,
    query: &'a Query,
}

/// Search among all data based on the input query
pub async fn search(db: &DbPool, query: &Query) -> Result<WordResult, Error> {
    let start = SystemTime::now();
    let search = Search { query, db };

    // Try to use cache
    if let Some(c_res) = search.get_cache().await {
        return Ok(c_res);
    }

    // Perform the search
    let results = search.do_search().await?;

    println!("search took {:?}", start.elapsed());
    search.save_cache(results.clone()).await;
    Ok(results)
}

impl<'a> Search<'a> {
    /// Do the search
    async fn do_search(&self) -> Result<WordResult, Error> {
        let word_results = match self.query.form {
            Form::KanjiReading(_) => kanji::by_reading(self).await?,
            _ => self.do_word_search().await?,
        };

        // Chain and map results into one result vector
        let kanji_results = kanji::load_word_kanji_info(&self, &word_results).await?;
        let kanji_items = kanji_results.len();

        return Ok(WordResult {
            items: Self::merge_words_with_kanji(word_results, kanji_results),
            contains_kanji: kanji_items > 0,
        });
    }

    /// Search by a word
    async fn do_word_search(&self) -> Result<Vec<Word>, Error> {
        // Perform searches asynchronously
        let (native_word_res, gloss_word_res): (Vec<Word>, Vec<Word>) =
            futures::try_join!(self.native_results(), self.gloss_results())?;

        // Chain native and word results into one vector
        Ok(native_word_res
            .into_iter()
            .chain(gloss_word_res)
            .collect_vec())
    }

    /// Perform a native word search
    async fn native_results(&self) -> Result<Vec<Word>, Error> {
        if self.query.language != QueryLang::Japanese {
            return Ok(vec![]);
        }

        // Define basic search structure
        let mut word_search = WordSearch::new(self.db, &self.query.query);
        word_search
            .with_language(self.query.settings.user_lang)
            .with_english_glosses(self.query.settings.show_english);

        if self.query.has_part_of_speech_tags() {
            word_search.with_pos_filter(&self.query.get_part_of_speech_tags());
        }

        // Perform the word search
        let mut wordresults =
            if real_string_len(&self.query.query) <= 2 && self.query.query.is_kana() {
                // Search for exact matches only if query.len() <= 2
                let res = word_search
                    .with_mode(SearchMode::Exact)
                    .with_language(self.query.settings.user_lang)
                    .search_native()
                    .await?;

                if res.is_empty() {
                    // Do another search if no exact result was found
                    word_search
                        .with_mode(SearchMode::RightVariable)
                        .search_native()
                        .await?
                } else {
                    res
                }
            } else {
                word_search
                    .with_mode(SearchMode::RightVariable)
                    .search_native()
                    .await?
            };

        // Sort the results based
        NativeWordOrder::new(&self.query.query).sort(&mut wordresults);

        // Limit search to 10 results
        wordresults.truncate(10);

        Ok(wordresults)
    }

    /// Search gloss readings
    async fn gloss_results(&self) -> Result<Vec<Word>, Error> {
        if !(self.query.language == QueryLang::Foreign
            || self.query.language == QueryLang::Undetected)
        {
            return Ok(vec![]);
        }

        let mode = if real_string_len(&self.query.query) < 4 {
            SearchMode::Exact
        } else {
            SearchMode::Variable
        };

        // TODO don't make exact search
        let mut word_search = WordSearch::new(self.db, &self.query.query);
        word_search
            .with_language(self.query.settings.user_lang)
            .with_case_insensitivity(true)
            .with_english_glosses(self.query.settings.show_english)
            .with_mode(mode);

        if self.query.has_part_of_speech_tags() {
            word_search.with_pos_filter(&self.query.get_part_of_speech_tags());
        }

        let mut wordresults = word_search.search_by_glosses().await?;

        // Sort the results based
        GlossWordOrder::new(&self.query.query).sort(&mut wordresults);

        // Limit search to 10 results
        wordresults.truncate(10);

        Ok(wordresults)
    }

    fn merge_words_with_kanji(words: Vec<Word>, kanji: Vec<DbKanji>) -> Vec<Item> {
        kanji
            .into_iter()
            .map(|i| i.into())
            .collect::<Vec<Item>>()
            .into_iter()
            .chain(words.into_iter().map(|i| i.into()).collect_vec())
            .collect_vec()
    }

    async fn get_cache(&self) -> Option<WordResult> {
        SEARCH_CACHE
            .lock()
            .await
            .cache_get(&self.query.get_hash())
            .map(|i| i.clone())
    }

    async fn save_cache(&self, result: WordResult) {
        SEARCH_CACHE
            .lock()
            .await
            .cache_set(self.query.get_hash(), result);
    }
}
