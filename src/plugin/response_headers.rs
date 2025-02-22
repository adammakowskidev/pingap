// Copyright 2024 Tree xie.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use super::{get_step_conf, get_str_slice_conf, Error, ResponsePlugin, Result};
use crate::config::{PluginCategory, PluginConf, PluginStep};
use crate::http_extra::{convert_header, convert_header_value, HttpHeader};
use crate::state::State;
use async_trait::async_trait;
use bytes::Bytes;
use http::header::HeaderName;
use log::debug;
use pingora::http::ResponseHeader;
use pingora::proxy::Session;
use std::str::FromStr;

pub struct ResponseHeaders {
    plugin_step: PluginStep,
    add_headers: Vec<HttpHeader>,
    remove_headers: Vec<HeaderName>,
    set_headers: Vec<HttpHeader>,
}

struct ResponseHeadersParams {
    plugin_step: PluginStep,
    add_headers: Vec<HttpHeader>,
    remove_headers: Vec<HeaderName>,
    set_headers: Vec<HttpHeader>,
}

impl TryFrom<&PluginConf> for ResponseHeadersParams {
    type Error = Error;
    fn try_from(value: &PluginConf) -> Result<Self> {
        let step = get_step_conf(value);

        let mut add_headers = vec![];
        for item in get_str_slice_conf(value, "add_headers").iter() {
            let header = convert_header(item).map_err(|e| Error::Invalid {
                category: PluginCategory::ResponseHeaders.to_string(),
                message: e.to_string(),
            })?;
            if let Some(item) = header {
                add_headers.push(item);
            }
        }

        let mut set_headers = vec![];
        for item in get_str_slice_conf(value, "set_headers").iter() {
            let header = convert_header(item).map_err(|e| Error::Invalid {
                category: PluginCategory::ResponseHeaders.to_string(),
                message: e.to_string(),
            })?;
            if let Some(item) = header {
                set_headers.push(item);
            }
        }
        let mut remove_headers = vec![];
        for item in get_str_slice_conf(value, "remove_headers").iter() {
            let item = HeaderName::from_str(item).map_err(|e| Error::Invalid {
                category: PluginCategory::ResponseHeaders.to_string(),
                message: e.to_string(),
            })?;
            remove_headers.push(item);
        }
        let params = Self {
            plugin_step: step,
            add_headers,
            set_headers,
            remove_headers,
        };

        if params.plugin_step != PluginStep::Response {
            return Err(Error::Invalid {
                category: PluginCategory::ResponseHeaders.to_string(),
                message: "Response headers plugin should be executed at response step".to_string(),
            });
        }
        Ok(params)
    }
}

impl ResponseHeaders {
    pub fn new(params: &PluginConf) -> Result<Self> {
        debug!("new stats proxy plugin, params:{params:?}");
        let params = ResponseHeadersParams::try_from(params)?;

        Ok(Self {
            plugin_step: params.plugin_step,
            add_headers: params.add_headers,
            remove_headers: params.remove_headers,
            set_headers: params.set_headers,
        })
    }
}

#[async_trait]
impl ResponsePlugin for ResponseHeaders {
    #[inline]
    fn step(&self) -> String {
        self.plugin_step.to_string()
    }
    #[inline]
    fn category(&self) -> PluginCategory {
        PluginCategory::ResponseHeaders
    }
    #[inline]
    async fn handle(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut State,
        upstream_response: &mut ResponseHeader,
    ) -> pingora::Result<Option<Bytes>> {
        if step != self.plugin_step {
            return Ok(None);
        }
        // add --> remove --> set
        // ingore error
        for (name, value) in &self.add_headers {
            if let Some(value) = convert_header_value(value, session, ctx) {
                let _ = upstream_response.append_header(name, value);
            } else {
                let _ = upstream_response.append_header(name, value);
            }
        }
        for name in &self.remove_headers {
            let _ = upstream_response.remove_header(name);
        }
        for (name, value) in &self.set_headers {
            if let Some(value) = convert_header_value(value, session, ctx) {
                let _ = upstream_response.insert_header(name, value);
            } else {
                let _ = upstream_response.insert_header(name, value);
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::{ResponseHeaders, ResponseHeadersParams};
    use crate::state::State;
    use crate::{config::PluginConf, config::PluginStep, plugin::ResponsePlugin};
    use pingora::http::ResponseHeader;
    use pingora::proxy::Session;
    use pretty_assertions::assert_eq;
    use tokio_test::io::Builder;

    #[test]
    fn test_response_headers_params() {
        let params = ResponseHeadersParams::try_from(
            &toml::from_str::<PluginConf>(
                r###"
step = "response"
add_headers = [
"X-Service:1",
"X-Service:2",
]
set_headers = [
"X-Response-Id:123"
]
remove_headers = [
"Content-Type"
]
"###,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!("response", params.plugin_step.to_string());
        assert_eq!(
            r#"[("x-service", "1"), ("x-service", "2")]"#,
            format!("{:?}", params.add_headers)
        );
        assert_eq!(
            r#"[("x-response-id", "123")]"#,
            format!("{:?}", params.set_headers)
        );
        assert_eq!(
            r#"["content-type"]"#,
            format!("{:?}", params.remove_headers)
        );

        let result = ResponseHeadersParams::try_from(
            &toml::from_str::<PluginConf>(
                r###"
add_headers = [
"X-Service:1",
"X-Service:2",
]
set_headers = [
"X-Response-Id:123"
]
remove_headers = [
"Content-Type"
]
"###,
            )
            .unwrap(),
        );
        assert_eq!("Plugin response_headers invalid, message: Response headers plugin should be executed at response step", result.err().unwrap().to_string());
    }

    #[tokio::test]
    async fn test_response_headers() {
        let response_headers = ResponseHeaders::new(
            &toml::from_str::<PluginConf>(
                r###"
step = "response"
add_headers = [
    "X-Service:1",
    "X-Service:2",
]
set_headers = [
    "X-Response-Id:123"
]
remove_headers = [
    "Content-Type"
]
    "###,
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!("response_headers", response_headers.category().to_string());
        assert_eq!("response", response_headers.step().to_string());

        let headers = ["Accept-Encoding: gzip"].join("\r\n");
        let input_header = format!("GET /vicanso/pingap?size=1 HTTP/1.1\r\n{headers}\r\n\r\n");
        let mock_io = Builder::new().read(input_header.as_bytes()).build();
        let mut session = Session::new_h1(Box::new(mock_io));
        session.read_request().await.unwrap();

        let mut upstream_response = ResponseHeader::build_no_case(200, None).unwrap();

        upstream_response
            .append_header("Content-Type", "application/json")
            .unwrap();

        response_headers
            .handle(
                PluginStep::Response,
                &mut session,
                &mut State::default(),
                &mut upstream_response,
            )
            .await
            .unwrap();

        assert_eq!(
            r###"ResponseHeader { base: Parts { status: 200, version: HTTP/1.1, headers: {"x-service": "1", "x-service": "2", "x-response-id": "123"} }, header_name_map: None, reason_phrase: None }"###,
            format!("{upstream_response:?}")
        )
    }
}
