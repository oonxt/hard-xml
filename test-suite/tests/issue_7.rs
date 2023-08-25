use chrono::offset::TimeZone;
use chrono::{DateTime, NaiveDate, Utc};
use hard_xml::{XmlRead, XmlResult, XmlWrite};

#[derive(Debug, PartialEq, XmlRead, XmlWrite)]
#[xml(tag = "document")]
struct Document {
    #[xml(attr = "datetime")]
    datetime: DateTime<Utc>,
}

#[test]
fn datetime_roundtrip() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(
        Document::from_str(r#"<document datetime="1970-01-01T00:00:00.0Z" />"#)?,
        Document {
            datetime: Utc.timestamp_opt(0, 0).unwrap()
        }
    );

    let datetime = NaiveDate::from_ymd_opt(2018, 1, 26).unwrap();
    let datetime = datetime
        .and_hms_micro_opt(18, 30, 9, 453_829)
        .unwrap()
        .and_utc();
    assert_eq!(
        (Document { datetime }).to_string()?,
        r#"<document datetime="2018-01-26 18:30:09.453829 UTC"/>"#
    );

    Ok(())
}
