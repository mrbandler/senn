//! The secrets backend protocol of Senn, version 1.
//!
//! A backend is a program that turns a secret reference into the secret. Senn starts it
//! once per secret, writes one [`Request`] as JSON to its stdin and reads one
//! [`Response`] as JSON from its stdout. On an error the backend exits non-zero and
//! writes a message to stderr, never the value.
//!
//! The full specification is in the book, chapter "Secrets backends".

use serde::{Deserialize, Serialize};

/// The protocol version this crate speaks. A backend rejects any other version.
pub const VERSION: u32 = 1;

/// A request from Senn to a backend.
///
/// On the wire, the fields of [`Action`] sit next to `protocol` in one flat JSON object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    /// Protocol version. A backend rejects versions it doesn't know.
    pub protocol: u32,

    /// What to do.
    #[serde(flatten)]
    pub action: Action,
}

/// The actions of protocol 1. The variant name is the `action` field on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    /// Read the secret at a reference, for example `op://vault/item/field`.
    Read {
        /// The secret reference, in the backend's own format.
        #[serde(rename = "ref")]
        reference: String,
    },
}

/// A successful response from a backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    /// The secret, exactly as stored: no added or removed characters, trailing newlines
    /// included.
    pub value: String,
}

impl Request {
    /// Builds a read request for `reference` with the current [`VERSION`].
    ///
    /// # Examples
    ///
    /// ```
    /// use senn_protocol::Request;
    ///
    /// let json = serde_json::to_string(&Request::read("op://vault/item/field")).unwrap();
    /// assert_eq!(json, r#"{"protocol":1,"action":"read","ref":"op://vault/item/field"}"#);
    /// ```
    #[must_use]
    pub fn read(reference: impl Into<String>) -> Self {
        Self {
            protocol: VERSION,
            action: Action::Read {
                reference: reference.into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_request_wire_format() {
        let json = serde_json::to_value(Request::read("op://v/i/f")).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"protocol": 1, "action": "read", "ref": "op://v/i/f"})
        );
    }

    #[test]
    fn unknown_action_is_rejected() {
        let result = serde_json::from_str::<Request>(r#"{"protocol":1,"action":"write","ref":"x"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn response_wire_format() {
        let response: Response = serde_json::from_str(r#"{"value":"a\nb\n"}"#).unwrap();
        assert_eq!(response.value, "a\nb\n");
    }
}
