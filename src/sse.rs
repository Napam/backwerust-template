use axum::http::{HeaderMap, header};
use axum::response::{IntoResponse, Response};

pub fn is_datastar_request(headers: &HeaderMap) -> bool {
    headers.contains_key("datastar-request")
}

/// Formats one `datastar-patch-elements` event that morphs the target matching
/// `selector` with `elements`. Every line of the HTML gets its own `data:
/// elements` field so the browser reassembles the markup exactly.
pub fn patch_elements(selector: &str, elements: &str) -> String {
    // SSE treats \r, \n and \r\n as line terminators, so split on all of them.
    // Otherwise a value with a bare carriage return could inject extra fields.
    let normalized = elements.replace("\r\n", "\n").replace('\r', "\n");
    let mut body = String::with_capacity(normalized.len() + 128);
    body.push_str("event: datastar-patch-elements\n");
    body.push_str("data: mode outer\n");
    body.push_str("data: selector ");
    body.push_str(selector);
    body.push('\n');
    for line in normalized.split('\n') {
        body.push_str("data: elements ");
        body.push_str(line);
        body.push('\n');
    }
    body.push('\n');
    body
}

pub fn patch_response(selector: &str, elements: &str) -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/event-stream"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        patch_elements(selector, elements),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::patch_elements;

    #[test]
    fn single_line_elements() {
        let body = patch_elements("#crud", "<section id=\"crud\">hi</section>");

        assert_eq!(
            body,
            concat!(
                "event: datastar-patch-elements\n",
                "data: mode outer\n",
                "data: selector #crud\n",
                "data: elements <section id=\"crud\">hi</section>\n",
                "\n",
            )
        );
    }

    #[test]
    fn every_line_of_multi_line_elements_is_a_field() {
        let body = patch_elements("#crud", "<section id=\"crud\">\n<p>one</p>\n</section>");

        assert_eq!(
            body,
            concat!(
                "event: datastar-patch-elements\n",
                "data: mode outer\n",
                "data: selector #crud\n",
                "data: elements <section id=\"crud\">\n",
                "data: elements <p>one</p>\n",
                "data: elements </section>\n",
                "\n",
            )
        );
    }

    #[test]
    fn blank_lines_inside_elements_do_not_end_the_event() {
        let body = patch_elements("#crud", "<p>one</p>\n\n<p>two</p>");

        assert_eq!(
            body,
            concat!(
                "event: datastar-patch-elements\n",
                "data: mode outer\n",
                "data: selector #crud\n",
                "data: elements <p>one</p>\n",
                "data: elements \n",
                "data: elements <p>two</p>\n",
                "\n",
            )
        );
        assert!(body.ends_with("\n\n"));
    }

    #[test]
    fn empty_elements_still_produce_a_terminated_event() {
        let body = patch_elements("#crud", "");

        assert_eq!(
            body,
            concat!(
                "event: datastar-patch-elements\n",
                "data: mode outer\n",
                "data: selector #crud\n",
                "data: elements \n",
                "\n",
            )
        );
    }

    #[test]
    fn trailing_newline_keeps_the_markup_intact() {
        let body = patch_elements("#crud", "<p>one</p>\n");

        assert!(body.starts_with(
            "event: datastar-patch-elements\ndata: mode outer\ndata: selector #crud\ndata: elements <p>one</p>\ndata: elements \n\n"
        ));
    }

    #[test]
    fn lone_carriage_return_does_not_start_a_new_event() {
        let body = patch_elements(
            "#crud",
            "<p>one</p>\r\revent: datastar-patch-elements\rdata: mode remove\rdata: selector body",
        );

        assert_eq!(
            body,
            concat!(
                "event: datastar-patch-elements\n",
                "data: mode outer\n",
                "data: selector #crud\n",
                "data: elements <p>one</p>\n",
                "data: elements \n",
                "data: elements event: datastar-patch-elements\n",
                "data: elements data: mode remove\n",
                "data: elements data: selector body\n",
                "\n",
            )
        );
    }

    #[test]
    fn crlf_lines_do_not_keep_a_carriage_return() {
        let body = patch_elements("#crud", "<p>one</p>\r\n<p>two</p>");

        assert_eq!(
            body,
            concat!(
                "event: datastar-patch-elements\n",
                "data: mode outer\n",
                "data: selector #crud\n",
                "data: elements <p>one</p>\n",
                "data: elements <p>two</p>\n",
                "\n",
            )
        );
    }
}
