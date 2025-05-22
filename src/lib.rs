#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Version {
    #[serde(rename = "1.0")]
    One,
    #[serde(rename = "2.0")]
    Two,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Header {
    pub jsonrpc: Version,
    pub id: Option<usize>,
}

impl Header {
    pub fn v1(id: Option<usize>) -> Self {
        Self {
            jsonrpc: Version::One,
            id,
        }
    }

    pub fn v2(id: Option<usize>) -> Self {
        Self {
            jsonrpc: Version::Two,
            id,
        }
    }
}

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct JsonRpcRequest<T> {
    #[serde(flatten)]
    pub header: Header,
    #[serde(flatten)]
    pub payload: T,
}

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct JsonRpcResponse<R, E>(pub JsonRpcRequest<Response<R, E>>);

impl<R, E> JsonRpcResponse<R, E> {
    pub fn result(header: Header, result: R) -> Self {
        Self(JsonRpcRequest {
            header,
            payload: Response::result(result),
        })
    }

    pub fn error(header: Header, error: E) -> Self {
        Self(JsonRpcRequest {
            header,
            payload: Response::error(error),
        })
    }
}

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Response<R, E> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<R>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<E>,
}

impl<R, E> Response<R, E> {
    pub fn result(result: R) -> Self {
        Self {
            result: Some(result),
            error: None,
        }
    }

    pub fn error(error: E) -> Self {
        Self {
            result: None,
            error: Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn test_version_serialization() {
        let version = Version::One;
        let serialized = serde_json::to_string(&version).unwrap();
        assert_eq!(serialized, "\"1.0\"");

        let version = Version::Two;
        let serialized = serde_json::to_string(&version).unwrap();
        assert_eq!(serialized, "\"2.0\"");
    }

    #[test]
    fn test_version_deserialization() {
        let deserialized: Version = serde_json::from_str("\"1.0\"").unwrap();
        assert_eq!(deserialized, Version::One);

        let deserialized: Version = serde_json::from_str("\"2.0\"").unwrap();
        assert_eq!(deserialized, Version::Two);
    }

    #[test]
    fn test_header_constructors() {
        let header = Header::v1(Some(123));
        assert_eq!(header.jsonrpc, Version::One);
        assert_eq!(header.id, Some(123));

        let header = Header::v1(None);
        assert_eq!(header.jsonrpc, Version::One);
        assert_eq!(header.id, None);

        let header = Header::v2(Some(456));
        assert_eq!(header.jsonrpc, Version::Two);
        assert_eq!(header.id, Some(456));

        let header = Header::v2(None);
        assert_eq!(header.jsonrpc, Version::Two);
        assert_eq!(header.id, None);
    }

    #[test]
    fn test_header_serialization() {
        let header = Header::v2(Some(42));
        let json = serde_json::to_value(&header).unwrap();
        assert_eq!(json, json!({"jsonrpc": "2.0", "id": 42}));

        let header = Header::v1(None);
        let json = serde_json::to_value(&header).unwrap();
        assert_eq!(json, json!({"jsonrpc": "1.0", "id": null}));
    }

    #[test]
    fn test_header_deserialization() {
        let json = r#"{"jsonrpc": "1.0", "id": 123}"#;
        let header: Header = serde_json::from_str(json).unwrap();
        assert_eq!(header, Header::v1(Some(123)));

        let json = r#"{"jsonrpc": "2.0", "id": null}"#;
        let header: Header = serde_json::from_str(json).unwrap();
        assert_eq!(header, Header::v2(None));
    }

    #[test]
    fn test_json_rpc_request() {
        #[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        struct TestPayload {
            method: String,
            params: Vec<String>,
        }

        let header = Header::v2(Some(1));
        let payload = TestPayload {
            method: "test".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
        };

        let request = JsonRpcRequest { header, payload };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            json,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "test",
                "params": ["a", "b"]
            })
        );

        let deserialized: JsonRpcRequest<TestPayload> = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, request);
    }

    #[test]
    fn test_json_rpc_response_result() {
        let header = Header::v2(Some(42));
        let result = "success".to_string();

        let response: JsonRpcResponse<String, ()> =
            JsonRpcResponse::result(header.clone(), result.clone());

        assert_eq!(response.0.header, header);
        assert_eq!(response.0.payload.result, Some(result.clone()));
        assert_eq!(response.0.payload.error, None);

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(
            json,
            json!({
                "jsonrpc": "2.0",
                "id": 42,
                "result": "success"
            })
        );

        let deserialized: JsonRpcResponse<String, Value> = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.0.header, header);
        assert_eq!(deserialized.0.payload.result, Some(result));
        assert_eq!(deserialized.0.payload.error, None);
    }

    #[test]
    fn test_json_rpc_response_error() {
        #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        struct TestError {
            code: i32,
            message: String,
        }

        let header = Header::v2(Some(42));
        let error = TestError {
            code: -32600,
            message: "Invalid Request".to_string(),
        };

        let response: JsonRpcResponse<(), TestError> =
            JsonRpcResponse::error(header.clone(), error.clone());

        assert_eq!(response.0.header, header);
        assert_eq!(response.0.payload.error, Some(error.clone()));
        assert_eq!(response.0.payload.result, None);

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(
            json,
            json!({
                "jsonrpc": "2.0",
                "id": 42,
                "error": {
                    "code": -32600,
                    "message": "Invalid Request"
                }
            })
        );

        let deserialized: JsonRpcResponse<Value, TestError> = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.0.header, header);
        assert_eq!(deserialized.0.payload.error, Some(error));
        assert_eq!(deserialized.0.payload.result, None);
    }

    #[test]
    fn test_response_constructors() {
        let response: Response<&str, ()> = Response::result("success");
        assert_eq!(response.result, Some("success"));
        assert_eq!(response.error, None);

        let response: Response<(), &str> = Response::error("error");
        assert_eq!(response.result, None);
        assert_eq!(response.error, Some("error"));
    }

    #[test]
    fn test_skip_serializing_none_fields() {
        let response: Response<&str, ()> = Response::result("success");
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json, json!({"result": "success"}));
        assert!(!json.as_object().unwrap().contains_key("error"));

        let response: Response<(), &str> = Response::error("error");
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json, json!({"error": "error"}));
        assert!(!json.as_object().unwrap().contains_key("result"));
    }
}
