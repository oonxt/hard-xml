hard-xml/README.md

hashmap support:

prefix:
```
    #[derive(XmlRead, XmlWrite, PartialEq, Debug)]
    #[xml(tag = "mapdata")]
    struct StringMapData {
        #[xml(prefix = "v")]
        pub s: HashMap<String, String>,
    }
    
    
    #[derive(XmlRead, XmlWrite, PartialEq, Debug)]
    #[xml(tag = "mapdata")]
    struct IntMapData {
        #[xml(prefix = "v")]
        pub i: HashMap<String, i32>,
    }

    let mut s = HashMap::new();
    s.insert("foo".to_string(), "1".to_string());
    s.insert("fbb".to_string(), "2".to_string());

    let map_data = StringMapData { s };

    assert_eq!(
        r#"<mapdata v:fbb="2" v:foo="1"/>"#,
        map_data.to_string()?
    );

    assert_eq!(
        StringMapData::from_str(r#"<mapdata v:foo="1" v:fbb="2"/>"#)?,
        map_data
    );

    let mut i = HashMap::new();
    i.insert("foo".to_string(), 1);
    i.insert("fbb".to_string(), 2);
    let map_data = IntMapData {i};

    assert_eq!(
        r#"<mapdata v:fbb="2" v:foo="1"/>"#,
        map_data.to_string()?
    );

    assert_eq!(
        IntMapData::from_str(r#"<mapdata v:foo="1" v:fbb="2"/>"#)?,
        map_data
    );

```
