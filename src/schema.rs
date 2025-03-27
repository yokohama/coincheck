// @generated automatically by Diesel CLI.

diesel::table! {
    optimized_mas (id) {
        id -> Int4,
        pair -> Text,
        short_ma -> Int4,
        long_ma -> Int4,
        offset_minutes -> Int4,
        win_rate_pct -> Nullable<Float8>,
        total -> Nullable<Int4>,
        wins -> Nullable<Int4>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    orders (id) {
        id -> Int4,
        rate -> Float8,
        crypto_amount -> Float8,
        #[max_length = 255]
        order_type -> Varchar,
        #[max_length = 255]
        pair -> Varchar,
        created_at -> Timestamp,
        buy_rate -> Nullable<Float8>,
        sell_rate -> Nullable<Float8>,
        spread_ratio -> Nullable<Float8>,
        jpy_amount -> Nullable<Float8>,
        api_error_msg -> Nullable<Text>,
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
    optimized_mas,
    orders,
    summaries,
    summary_records,
    tickers,
    transactions,
);
