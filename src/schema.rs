// @generated automatically by Diesel CLI.

diesel::table! {
    orders (id) {
        id -> Int4,
        rate -> Float8,
        amount -> Float8,
        #[max_length = 255]
        order_type -> Varchar,
        #[max_length = 255]
        pair -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::table! {
    tickers (id) {
        id -> Int4,
        pair -> Text,
        last -> Float8,
        bid -> Float8,
        ask -> Float8,
        high -> Float8,
        low -> Float8,
        volume -> Float8,
        timestamp -> Nullable<Timestamp>,
    }
}

diesel::table! {
    transactions (id) {
        id -> Int4,
        order_id -> Int4,
        created_at -> Timestamp,
        rate -> Float8,
        amount -> Float8,
        #[max_length = 255]
        order_type -> Varchar,
        #[max_length = 255]
        pair -> Varchar,
        price -> Float8,
        #[max_length = 255]
        fee_currency -> Varchar,
        fee -> Float8,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    orders,
    tickers,
    transactions,
);
