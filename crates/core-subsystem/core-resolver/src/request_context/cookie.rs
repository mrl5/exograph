use std::collections::HashMap;

use crate::request_context::{ParsedContext, RequestContext};
use async_trait::async_trait;
use cookie::Cookie;
use serde_json::Value;

use super::{BoxedParsedContext, ContextParsingError, Request};

pub struct CookieExtractor;

impl CookieExtractor {
    pub fn parse_context(request: &dyn Request) -> Result<BoxedParsedContext, ContextParsingError> {
        let cookie_headers = request.get_headers("cookie");

        let cookie_strings = cookie_headers
            .into_iter()
            .map(|header| header.split(';').collect());

        let cookies = cookie_strings
            .map(|cookie_string: String| {
                Cookie::parse(cookie_string)
                    .map(|cookie| (cookie.name().to_owned(), cookie))
                    .map_err(|_| ContextParsingError::Malformed)
            })
            .collect::<Result<Vec<(String, Cookie)>, ContextParsingError>>()?;

        let cookie_map: HashMap<String, Cookie> = cookies.into_iter().collect();

        Ok(Box::new(ParsedCookieContext {
            cookies: cookie_map,
        }))
    }
}

pub struct ParsedCookieContext {
    cookies: HashMap<String, Cookie<'static>>,
}

#[async_trait]
impl ParsedContext for ParsedCookieContext {
    fn annotation_name(&self) -> &str {
        "cookie"
    }

    async fn extract_context_field<'r>(
        &self,
        key: Option<&str>,
        _request_context: &'r RequestContext<'r>,
        _request: &'r (dyn Request + Send + Sync),
    ) -> Option<Value> {
        self.cookies
            .get(key?)
            .map(|c| (*c.value()).to_string().into())
    }
}