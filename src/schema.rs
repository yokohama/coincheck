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
        buy_rate -> Nullable<Float8>,
        sell_rate -> Nullable<Float8>,
        spread_ratio -> Nullable<Float8>,
    }
}

diesel::table! {
    summaries (id) {
        id -> Int4,
        total_invested -> Float8,
        total_jpy_value -> Float8,
        pl -> Float8,
        created_at -> Timestamp,
    }
}

diesel::table! {
    summary_records (id) {
        id -> Int4,
        summary_id -> Int4,
        #[max_length = 255]
        currency -> Varchar,
        amount -> Float8,
        rate -> Float8,
        jpy_value -> Float8,
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
    summaries,
    summary_records,
    tickers,
    transactions,
);
