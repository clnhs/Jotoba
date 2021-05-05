table! {
    dict (id) {
        id -> Int4,
        sequence -> Int4,
        reading -> Text,
        kanji -> Bool,
        no_kanji -> Bool,
        priorities -> Nullable<Array<Text>>,
        information -> Nullable<Array<Int4>>,
        kanji_info -> Nullable<Array<Int4>>,
        jlpt_lvl -> Nullable<Int4>,
        is_main -> Bool,
        accents -> Nullable<Array<Int4>>,
    }
}

table! {
    kanji (id) {
        id -> Int4,
        literal -> Bpchar,
        meaning -> Array<Text>,
        grade -> Nullable<Int4>,
        stroke_count -> Int4,
        frequency -> Nullable<Int4>,
        jlpt -> Nullable<Int4>,
        variant -> Nullable<Array<Text>>,
        onyomi -> Nullable<Array<Text>>,
        kunyomi -> Nullable<Array<Text>>,
        chinese -> Nullable<Text>,
        korean_r -> Nullable<Array<Text>>,
        korean_h -> Nullable<Array<Text>>,
        natori -> Nullable<Array<Text>>,
        kun_dicts -> Nullable<Array<Int4>>,
    }
}

table! {
    name (id) {
        id -> Int4,
        sequence -> Int4,
        kana -> Text,
        kanji -> Nullable<Text>,
        transcription -> Text,
        name_type -> Nullable<Array<Int4>>,
        xref -> Nullable<Text>,
    }
}

table! {
    sense (id) {
        id -> Int4,
        sequence -> Int4,
        language -> Int4,
        gloss_pos -> Int4,
        gloss -> Text,
        misc -> Nullable<Text>,
        part_of_speech -> Nullable<Array<Text>>,
        dialect -> Nullable<Text>,
        xref -> Nullable<Text>,
        gtype -> Nullable<Int4>,
        field -> Nullable<Text>,
        information -> Nullable<Text>,
        antonym -> Nullable<Text>,
        pos_simplified -> Nullable<Array<Int4>>,
    }
}

table! {
    sentence (id) {
        id -> Int4,
        content -> Text,
        furigana -> Text,
    }
}

table! {
    sentence_translation (id) {
        id -> Int4,
        sentence_id -> Int4,
        language -> Int4,
        content -> Text,
    }
}

table! {
    sentence_vocabulary (id) {
        id -> Int4,
        sentence_id -> Int4,
        dict_sequence -> Int4,
        start -> Int4,
    }
}

joinable!(sentence_translation -> sentence (sentence_id));
joinable!(sentence_vocabulary -> sentence (sentence_id));

allow_tables_to_appear_in_same_query!(
    dict,
    kanji,
    name,
    sense,
    sentence,
    sentence_translation,
    sentence_vocabulary,
);
