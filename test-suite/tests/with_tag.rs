use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;

mod withmod {
    use hard_xml::{XmlError, XmlResult};
    use std::borrow::Cow;
    use std::error::Error;
    use std::fmt::Display;
    use std::str::FromStr;

    #[inline]
    pub fn to_xml<T>(s: &str) -> XmlResult<T>
    where
        T: FromStr,
        <T as FromStr>::Err: Error + Send + Sync + 'static,
    {
        Ok(T::from_str(s).map_err(|err| XmlError::FromStr(Box::new(err)))?)
    }

    #[inline]
    pub fn from_xml<'a, U>(xmlval: &'a &'a U) -> Cow<'a, str>
    where
        U: Display + FromStr,
        <U as FromStr>::Err: Error + Send + Sync,
    {
        // reverse the string to show that it is not the default behaviour
        xmlval.to_string().chars().rev().collect::<String>().into()
    }
}

use hard_xml::{XmlRead, XmlResult, XmlWrite};

#[derive(XmlRead, XmlWrite, PartialEq, Debug)]
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
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(
        Withtag::from_str(r#"<withtag att1="att1"/>"#)?,
        Withtag {
            att1: String::from("att1"),
        }
    );

    // match with a reversed string in from_xml of withmod.
    assert_eq!(
        (Withtag {
            att1: String::from("att1"),
        })
        .to_string()?,
        r#"<withtag att1="1tta"/>"#,
    );

    Ok(())
}
