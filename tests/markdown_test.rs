use mdreader::markdown;

#[test]
fn test_parse_links() {
    let content = "Check out [this link](https://example.com) here";
    let parsed = markdown::parse(content);
    assert_eq!(parsed.links.len(), 1);
    assert_eq!(parsed.links[0].text, "this link");
    assert_eq!(parsed.links[0].url, "https://example.com");
}
