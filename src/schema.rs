// @generated automatically by Diesel CLI.

diesel::table! {
    rule_content (id) {
        id -> Nullable<Text>,
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
        id -> Nullable<Text>,
        name -> Text,
        slug -> Text,
        description -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    rules (id) {
        id -> Nullable<Text>,
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
        id -> Nullable<Text>,
        rule_set_id -> Text,
        version_name -> Text,
        effective_from -> Date,
        effective_to -> Nullable<Date>,
        description -> Nullable<Text>,
        is_current -> Bool,
        created_at -> Timestamp,
    }
}

diesel::joinable!(rule_content -> rules (rule_id));
diesel::joinable!(rules -> rule_sets (rule_set_id));
diesel::joinable!(rules -> versions (version_id));
diesel::joinable!(versions -> rule_sets (rule_set_id));

diesel::allow_tables_to_appear_in_same_query!(
    rule_content,
    rule_sets,
    rules,
    versions,
);
