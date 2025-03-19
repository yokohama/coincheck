use serde::Deserialize;
use chrono::{NaiveDateTime, Utc, TimeZone};
use serde::Serializer;
use serde::Deserializer;

pub fn serialize_naive_datetime<S>(datetime: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    serializer.serialize_str(&s)
}

pub fn deserialize_naive_datetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").map_err(serde::de::Error::custom)
}

// `i64` の UNIX タイムスタンプを `NaiveDateTime` に変換
pub fn deserialize_unix_timestamp<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamp = i64::deserialize(deserializer)?;
    Ok(Utc.timestamp_opt(timestamp, 0).single().unwrap().naive_utc())
}

// `NaiveDateTime` を `i64` の UNIX タイムスタンプに変換
pub fn serialize_unix_timestamp<S>(datetime: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i64(datetime.and_utc().timestamp())
}
