// @generated automatically by Diesel CLI.

diesel::table! {
    glossary_content (id) {
        id -> Text,
        term_id -> Text,
        language -> Text,
        term -> Text,
        definition_markdown -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    glossary_terms (id) {
        id -> Text,
        rule_set_id -> Text,
        version_id -> Text,
        slug -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    rule_content (id) {
        id -> Text,
        rule_id -> Text,
        language -> Text,
        content_markdown -> Text,
        source_content_id -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    rule_sets (id) {
        id -> Text,
        name -> Text,
        slug -> Text,
        description -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    rules (id) {
        id -> Text,
        slug -> Text,
        rule_set_id -> Text,
        version_id -> Text,
        parent_rule_id -> Nullable<Text>,
        number -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    versions (id) {
        id -> Text,
        rule_set_id -> Text,
        version_name -> Text,
        effective_from -> Date,
        effective_to -> Nullable<Date>,
        description -> Nullable<Text>,
        is_current -> Bool,
        created_at -> Timestamp,
    }
}

diesel::joinable!(glossary_content -> glossary_terms (term_id));
diesel::joinable!(glossary_terms -> rule_sets (rule_set_id));
diesel::joinable!(glossary_terms -> versions (version_id));
diesel::joinable!(rule_content -> rules (rule_id));
diesel::joinable!(rules -> rule_sets (rule_set_id));
diesel::joinable!(rules -> versions (version_id));
diesel::joinable!(versions -> rule_sets (rule_set_id));

diesel::allow_tables_to_appear_in_same_query!(
    glossary_content,
    glossary_terms,
    rule_content,
    rule_sets,
    rules,
    versions,
);
