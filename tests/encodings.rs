use encoding_rs::{UTF_16BE, UTF_16LE, UTF_8, WINDOWS_1251};
use pretty_assertions::assert_eq;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event::*};
use quick_xml::reader::Reader;

mod decode {
    use super::*;
    use pretty_assertions::assert_eq;
    use quick_xml::encoding::detect_encoding;

    static UTF16BE_TEXT_WITH_BOM: &[u8] = include_bytes!("documents/encoding/utf16be-bom.xml");
    static UTF16LE_TEXT_WITH_BOM: &[u8] = include_bytes!("documents/encoding/utf16le-bom.xml");
    static UTF8_TEXT_WITH_BOM: &[u8] = include_bytes!("documents/encoding/utf8-bom.xml");

    static UTF8_TEXT: &str = r#"<?xml version="1.0"?>
<project name="project-name">
</project>
"#;

    #[test]
    fn test_detect_encoding() {
        // No BOM
        assert_eq!(detect_encoding(UTF8_TEXT.as_bytes()), Some((UTF_8, 0)));
        // BOM
        assert_eq!(detect_encoding(UTF8_TEXT_WITH_BOM), Some((UTF_8, 3)));
        assert_eq!(detect_encoding(UTF16BE_TEXT_WITH_BOM), Some((UTF_16BE, 2)));
        assert_eq!(detect_encoding(UTF16LE_TEXT_WITH_BOM), Some((UTF_16LE, 2)));
    }
}

#[test]
fn test_koi8_r_encoding() {
    let src = include_bytes!("documents/opennews_all.rss").as_ref();
    let mut buf = vec![];
    let mut r = Reader::from_reader(src);
    r.config_mut().trim_text(true);
    loop {
        match r.read_event_into(&mut buf) {
            Ok(Text(e)) => {
                e.decode().unwrap();
            }
            Ok(Eof) => break,
            _ => (),
        }
    }
}

/// Test data generated by helper project `test-gen`, which requires checkout of
/// an `encoding` submodule
mod detect {
    use super::*;
    use encoding_rs::*;
    use pretty_assertions::assert_eq;

    macro_rules! assert_matches {
        ($number:literal : $left:expr, $pattern:pat_param) => {{
            let event = $left;
            if !matches!(event, $pattern) {
                assert_eq!(
                    format!("{:#?}", event),
                    stringify!($pattern),
                    concat!("Message ", stringify!($number), " is incorrect")
                );
            }
        }};
    }
    macro_rules! check_detection {
        ($test:ident, $enc:ident, $file:literal) => {
            #[test]
            fn $test() {
                let mut r = Reader::from_reader(
                    include_bytes!(concat!("documents/encoding/", $file, ".xml")).as_ref(),
                );
                assert_eq!(r.decoder().encoding(), UTF_8);

                let mut buf = Vec::new();
                // XML declaration with encoding
                assert_matches!(1: r.read_event_into(&mut buf).unwrap(), Decl(_));
                assert_eq!(r.decoder().encoding(), $enc);
                assert_matches!(2: r.read_event_into(&mut buf).unwrap(), Text(_)); // spaces
                buf.clear();

                // Comment with information that this is generated file
                assert_matches!(3: r.read_event_into(&mut buf).unwrap(), Comment(_));
                assert_eq!(r.decoder().encoding(), $enc);
                assert_matches!(4: r.read_event_into(&mut buf).unwrap(), Text(_)); // spaces
                buf.clear();

                // Open root element tag. Contains 3 attributes:
                // - attribute1 - double-quoted. Value - all possible characters in that encoding
                // - attribute2 - single-quoted. Value - all possible characters in that encoding
                // - unquoted. Name and value - all possible characters in that encoding
                assert_matches!(5: r.read_event_into(&mut buf).unwrap(), Start(_));
                assert_eq!(r.decoder().encoding(), $enc);
                assert_matches!(6: r.read_event_into(&mut buf).unwrap(), Text(_)); // spaces
                buf.clear();

                // Processing instruction with all possible characters in that encoding
                assert_matches!(7: r.read_event_into(&mut buf).unwrap(), PI(_));
                assert_eq!(r.decoder().encoding(), $enc);
                assert_matches!(8: r.read_event_into(&mut buf).unwrap(), Text(_)); // spaces
                buf.clear();

                // Comment with all possible characters in that encoding
                assert_matches!(9: r.read_event_into(&mut buf).unwrap(), Comment(_));
                assert_eq!(r.decoder().encoding(), $enc);
                buf.clear();

                // Text with all possible characters in that encoding except some
                assert_matches!(10: r.read_event_into(&mut buf).unwrap(), Text(_));
                assert_eq!(r.decoder().encoding(), $enc);
                buf.clear();

                // Empty tag with name from all possible characters in that encoding except some
                assert_matches!(11: r.read_event_into(&mut buf).unwrap(), Empty(_));
                assert_eq!(r.decoder().encoding(), $enc);
                assert_matches!(12: r.read_event_into(&mut buf).unwrap(), Text(_)); // spaces
                buf.clear();

                // CDATA section with all possible characters in that encoding
                assert_matches!(13: r.read_event_into(&mut buf).unwrap(), CData(_));
                assert_eq!(r.decoder().encoding(), $enc);
                assert_matches!(14: r.read_event_into(&mut buf).unwrap(), Text(_)); // spaces
                buf.clear();

                // Close root element tag
                assert_matches!(15: r.read_event_into(&mut buf).unwrap(), End(_));
                assert_eq!(r.decoder().encoding(), $enc);
                buf.clear();

                // Document should end
                assert_matches!(16: r.read_event_into(&mut buf).unwrap(), Eof);
                assert_eq!(r.decoder().encoding(), $enc);
            }
        };
    }
    macro_rules! detect_test {
        ($test:ident, $enc:ident, $file:literal $($break:stmt)?) => {
            #[test]
            fn $test() {
                let mut r = Reader::from_reader(
                    include_bytes!(concat!("documents/encoding/", $file, ".xml")).as_ref(),
                );
                assert_eq!(r.decoder().encoding(), UTF_8);

                let mut buf = Vec::new();
                loop {
                    match dbg!(r.read_event_into(&mut buf).unwrap()) {
                        Eof => break,
                        _ => {}
                    }
                    assert_eq!(r.decoder().encoding(), $enc);
                    buf.clear();
                    $($break)?
                }
            }
        };
    }

    // Without BOM
    detect_test!(utf8, UTF_8, "utf8");
    detect_test!(utf16be, UTF_16BE, "utf16be");
    detect_test!(utf16le, UTF_16LE, "utf16le");

    // With BOM
    detect_test!(utf8_bom, UTF_8, "utf8-bom");
    detect_test!(utf16be_bom, UTF_16BE, "utf16be-bom");
    detect_test!(utf16le_bom, UTF_16LE, "utf16le-bom");

    // legacy multi-byte encodings (7)
    check_detection!(big5, BIG5, "Big5");
    check_detection!(euc_jp, EUC_JP, "EUC-JP");
    check_detection!(euc_kr, EUC_KR, "EUC-KR");
    check_detection!(gb18030, GB18030, "gb18030");
    check_detection!(gbk, GBK, "GBK");
    // TODO: XML in this encoding cannot be parsed successfully until #158 resolves
    // We only read the first event to ensure, that encoding detected correctly
    detect_test!(iso_2022_jp, ISO_2022_JP, "ISO-2022-JP" break);
    check_detection!(shift_jis, SHIFT_JIS, "Shift_JIS");

    // legacy single-byte encodings (19)
    check_detection!(ibm866, IBM866, "IBM866");
    check_detection!(iso_8859_2, ISO_8859_2, "ISO-8859-2");
    check_detection!(iso_8859_3, ISO_8859_3, "ISO-8859-3");
    check_detection!(iso_8859_4, ISO_8859_4, "ISO-8859-4");
    check_detection!(iso_8859_5, ISO_8859_5, "ISO-8859-5");
    check_detection!(iso_8859_6, ISO_8859_6, "ISO-8859-6");
    check_detection!(iso_8859_7, ISO_8859_7, "ISO-8859-7");
    check_detection!(iso_8859_8, ISO_8859_8, "ISO-8859-8");
    check_detection!(iso_8859_8_i, ISO_8859_8_I, "ISO-8859-8-I");
    check_detection!(iso_8859_10, ISO_8859_10, "ISO-8859-10");
    check_detection!(iso_8859_13, ISO_8859_13, "ISO-8859-13");
    check_detection!(iso_8859_14, ISO_8859_14, "ISO-8859-14");
    check_detection!(iso_8859_15, ISO_8859_15, "ISO-8859-15");
    check_detection!(iso_8859_16, ISO_8859_16, "ISO-8859-16");
    check_detection!(koi8_r, KOI8_R, "KOI8-R");
    check_detection!(koi8_u, KOI8_U, "KOI8-U");
    check_detection!(macintosh, MACINTOSH, "macintosh");
    check_detection!(windows_874, WINDOWS_874, "windows-874");
    check_detection!(windows_1250, WINDOWS_1250, "windows-1250");
    check_detection!(windows_1251, WINDOWS_1251, "windows-1251");
    check_detection!(windows_1252, WINDOWS_1252, "windows-1252");
    check_detection!(windows_1253, WINDOWS_1253, "windows-1253");
    check_detection!(windows_1254, WINDOWS_1254, "windows-1254");
    check_detection!(windows_1255, WINDOWS_1255, "windows-1255");
    check_detection!(windows_1256, WINDOWS_1256, "windows-1256");
    check_detection!(windows_1257, WINDOWS_1257, "windows-1257");
    check_detection!(windows_1258, WINDOWS_1258, "windows-1258");
    check_detection!(x_mac_cyrillic, X_MAC_CYRILLIC, "x-mac-cyrillic");
    check_detection!(x_user_defined, X_USER_DEFINED, "x-user-defined");
}

#[test]
fn bom_removed_from_initial_text() {
    let mut r =
        Reader::from_str("\u{FEFF}asdf<paired attr1=\"value1\" attr2=\"value2\">text</paired>");

    assert_eq!(r.read_event().unwrap(), Text(BytesText::new("asdf")));
    assert_eq!(
        r.read_event().unwrap(),
        Start(BytesStart::from_content(
            "paired attr1=\"value1\" attr2=\"value2\"",
            6
        ))
    );
    assert_eq!(r.read_event().unwrap(), Text(BytesText::new("text")));
    assert_eq!(r.read_event().unwrap(), End(BytesEnd::new("paired")));
    assert_eq!(r.read_event().unwrap(), Eof);
}

/// Checks that encoding is detected by BOM and changed after XML declaration
/// BOM indicates UTF-16LE, but XML - windows-1251
#[test]
fn bom_overridden_by_declaration() {
    let mut reader = Reader::from_reader(b"\xFF\xFE<?xml encoding='windows-1251'?>".as_ref());
    let mut buf = Vec::new();

    assert_eq!(reader.decoder().encoding(), UTF_8);
    assert!(matches!(reader.read_event_into(&mut buf).unwrap(), Decl(_)));
    assert_eq!(reader.decoder().encoding(), WINDOWS_1251);

    assert_eq!(reader.read_event_into(&mut buf).unwrap(), Eof);
}

/// Checks that encoding is changed by XML declaration, but only once
#[test]
fn only_one_declaration_changes_encoding() {
    let mut reader =
        Reader::from_reader(b"<?xml encoding='UTF-16'?><?xml encoding='windows-1251'?>".as_ref());
    let mut buf = Vec::new();

    assert_eq!(reader.decoder().encoding(), UTF_8);
    assert!(matches!(reader.read_event_into(&mut buf).unwrap(), Decl(_)));
    assert_eq!(reader.decoder().encoding(), UTF_16LE);

    assert!(matches!(reader.read_event_into(&mut buf).unwrap(), Decl(_)));
    assert_eq!(reader.decoder().encoding(), UTF_16LE);

    assert_eq!(reader.read_event_into(&mut buf).unwrap(), Eof);
}

/// Checks that XML declaration cannot change the encoding from UTF-8 if
/// a `Reader` was created using `from_str` method
#[test]
fn str_always_has_utf8() {
    let mut reader = Reader::from_str("<?xml encoding='UTF-16'?>");

    assert_eq!(reader.decoder().encoding(), UTF_8);
    reader.read_event().unwrap();
    assert_eq!(reader.decoder().encoding(), UTF_8);

    assert_eq!(reader.read_event().unwrap(), Eof);
}
