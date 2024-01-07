use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;

mod withmod {
    use std::borrow::Cow;
    use std::error::Error;
    use std::fmt::Display;
    use std::str::FromStr;

    pub fn from_xml<T>(s: &str) -> hard_xml::XmlResult<T>
    where
        T: FromStr,
        <T as FromStr>::Err: Error + Send + Sync + 'static,
    {
        Ok(T::from_str(&s.chars().rev().collect::<String>())
            .map_err(|err| hard_xml::XmlError::FromStr(Box::new(err)))?)
    }

    pub fn to_xml(xmlval: &impl Display) -> Cow<str> {
        // reverse the string to show that it is not the default behaviour
        xmlval.to_string().chars().rev().collect::<String>().into()
    }
}

#[derive(hard_xml::XmlRead, hard_xml::XmlWrite, PartialEq, Debug)]
#[xml(tag = "withtag")]
struct Withtag<U>
where
    U: Display + FromStr,
    // This bound is required because we need to wrap
    // the error with a `Box<dyn Error>`
    <U as FromStr>::Err: 'static + Error + Send + Sync,
{
    #[xml(attr = "att1", with = "withmod")]
    att1: U,
}

#[test]
fn test() -> hard_xml::XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(
        <Withtag<String> as hard_xml::XmlRead>::from_str(r#"<withtag att1="att1"/>"#)?,
        Withtag {
            att1: String::from("1tta"),
        }
    );

    // match with a reversed string in from_xml of withmod.
    assert_eq!(
        hard_xml::XmlWrite::to_string(&Withtag {
            att1: String::from("att1"),
        })?,
        r#"<withtag att1="1tta"/>"#,
    );

    Ok(())
}
