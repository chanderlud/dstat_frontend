table! {
    logs (time) {
        time -> Integer,
        server_name -> Text,
        rps -> Integer,
    }
}

table! {
    servers (server_id) {
        server_id -> Text,
        category -> Text,
        server_name -> Text,
        url -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    logs,
    servers,
);
