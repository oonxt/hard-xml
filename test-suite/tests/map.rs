use hard_xml::{XmlRead, XmlResult, XmlWrite};
use std::collections::HashMap;

#[derive(XmlRead, XmlWrite, PartialEq, Debug)]
#[xml(tag = "mapdata")]
struct MapData {
    #[xml(prefix = "v")]
    pub v: HashMap<String, i32>,
}


#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();
    let mut v = HashMap::new();
    v.insert("foo".to_string(), 1);
    v.insert("fbb".to_string(), 2);

    let map_data = MapData { v };

    assert_eq!(
        r#"<mapdata v:fbb="2" v:foo="1"/>"#,
        map_data.to_string()?
    );

    Ok(())
}
