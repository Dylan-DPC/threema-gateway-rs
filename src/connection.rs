//! Send and receive messages.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::io::Read;

use reqwest::{Client, StatusCode};
use reqwest::header::{Accept, ContentType};
use reqwest::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use data_encoding::{HEXLOWER, HEXLOWER_PERMISSIVE};

use ::crypto::EncryptedMessage;
use ::errors::ApiError;
use ::MSGAPI_URL;


/// A blob ID. Must contain exactly 16 lowercase hexadecimal characters.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlobId(pub [u8; 16]);

impl BlobId {
    /// Create a new BlobId.
    pub fn new(id: [u8; 16]) -> Self {
        BlobId(id)
    }

    /// Create a new BlobId from a 32 character hexadecimal String.
    pub fn from_str(id: &str) -> Result<Self, ApiError> {
        let bytes = HEXLOWER_PERMISSIVE.decode(id.as_bytes()).map_err(|_| ApiError::BadBlobId)?;
        if bytes.len() != 16 {
            return Err(ApiError::BadBlobId);
        }
        let mut arr = [0; 16];
        for i in 0..bytes.len() {
            arr[i] = bytes[i];
        }
        Ok(BlobId(arr))
    }
}

impl fmt::Display for BlobId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", HEXLOWER.encode(&self.0))
    }
}

/// Map HTTP response status code to an ApiError if it isn't "200".
///
/// Optionally, you can pass in the meaning of a 400 response code.
pub fn map_response_code(status: &StatusCode, bad_request_meaning: Option<ApiError>)
                         -> Result<(), ApiError> {
    match *status {
        // 200
        StatusCode::Ok => Ok(()),
        // 400
        StatusCode::BadRequest => match bad_request_meaning {
            Some(error) => Err(error),
            None => Err(ApiError::Other(format!("Bad response status code: {}", StatusCode::BadRequest))),
        },
        // 401
        StatusCode::Unauthorized => Err(ApiError::BadCredentials),
        // 402
        StatusCode::PaymentRequired => Err(ApiError::NoCredits),
        // 404
        StatusCode::NotFound => Err(ApiError::IdNotFound),
        // 413
        StatusCode::PayloadTooLarge => Err(ApiError::MessageTooLong),
        // 500
        StatusCode::InternalServerError => Err(ApiError::ServerError),
        e @ _ => Err(ApiError::Other(format!("Bad response status code: {}", e))),
    }
}

/// Different ways to specify a message recipient in basic mode.
#[derive(Debug)]
pub enum Recipient<'a> {
    /// Recipient identity (8 characters)
    Id(Cow<'a, str>),
    /// Recipient phone number (E.164), without leading +
    Phone(Cow<'a, str>),
    /// Recipient e-mail address
    Email(Cow<'a, str>),
}

impl<'a> Recipient<'a> {
    pub fn new_id<T: Into<Cow<'a, str>>>(id: T) -> Self {
        Recipient::Id(id.into())
    }

    pub fn new_phone<T: Into<Cow<'a, str>>>(phone: T) -> Self {
        Recipient::Phone(phone.into())
    }

    pub fn new_email<T: Into<Cow<'a, str>>>(email: T) -> Self {
        Recipient::Email(email.into())
    }
}

/// Send a message to the specified recipient in basic mode.
pub fn send_simple(from: &str, to: &Recipient, secret: &str, text: &str) -> Result<String, ApiError> {

    let client = Client::new().expect("Could not initialize HTTP client");

    // Check text length (max 3500 bytes)
    // Note: Strings in Rust are UTF8, so len() returns the byte count.
    if text.len() > 3500 {
        return Err(ApiError::MessageTooLong);
    }

    // Prepare POST data
    let mut params = HashMap::new();
    params.insert("from", from);
    params.insert("text", text);
    params.insert("secret", secret);
    match *to {
        Recipient::Id(ref id) => params.insert("to", id),
        Recipient::Phone(ref phone) => params.insert("phone", phone),
        Recipient::Email(ref email) => params.insert("email", email),
    };

    // Send request
    let mut res = try!(client.post(&format!("{}/send_simple", MSGAPI_URL))
        .form(&params)
        .header(Accept::json())
        .send());
    try!(map_response_code(res.status(), Some(ApiError::BadSenderOrRecipient)));

    // Read and return response body
    let mut body = String::new();
    try!(res.read_to_string(&mut body));

    Ok(body)
}

/// Send an encrypted E2E message to the specified recipient.
pub fn send_e2e(from: &str,
                to: &str,
                secret: &str,
                nonce: &[u8],
                ciphertext: &[u8],
                additional_params: Option<HashMap<String, String>>)
                -> Result<String, ApiError> {
    let client = Client::new().expect("Could not initialize HTTP client");

    // Prepare POST data
    let mut params = match additional_params {
        Some(p) => p,
        None => HashMap::new(),
    };
    params.insert("from".into(), from.into());
    params.insert("to".into(), to.into());
    params.insert("secret".into(), secret.into());
    params.insert("nonce".into(), HEXLOWER.encode(nonce));
    params.insert("box".into(), HEXLOWER.encode(ciphertext));

    // Send request
    let mut res = try!(client.post(&format!("{}/send_e2e", MSGAPI_URL))
        .form(&params)
        .header(Accept::json())
        .send());
    try!(map_response_code(res.status(), Some(ApiError::BadSenderOrRecipient)));

    // Read and return response body
    let mut body = String::new();
    try!(res.read_to_string(&mut body));

    Ok(body)
}

/// Upload a blob to the blob server.
pub fn blob_upload(from: &str, secret: &str, data: &EncryptedMessage) -> Result<BlobId, ApiError> {
    let client = Client::new().expect("Could not initialize HTTP client");

    // Build URL
    let url = format!("{}/upload_blob?from={}&secret={}", MSGAPI_URL, from, secret);

    // Build multipart/form-data request body
    let boundary = "3ma-d84f64f5-a138-4b0a-9e25-339257990c81-3ma".to_string();
    let mut req_body = Vec::new();
    req_body.extend_from_slice("--".as_bytes());
    req_body.extend_from_slice(&boundary.as_bytes());
    req_body.extend_from_slice("\r\n".as_bytes());
    req_body.extend_from_slice("Content-Disposition: form-data; name=\"blob\"\r\n".as_bytes());
    req_body.extend_from_slice("Content-Type: application/octet-stream\r\n\r\n".as_bytes());
    req_body.extend_from_slice(&data.ciphertext);
    req_body.extend_from_slice("\r\n--".as_bytes());
    req_body.extend_from_slice(&boundary.as_bytes());
    req_body.extend_from_slice("--\r\n".as_bytes());

    // Send request
    let mimetype = Mime(TopLevel::Multipart,
                        SubLevel::FormData,
                        vec![(Attr::Boundary, Value::Ext(boundary))]);
    let mut res = client.post(&url)
        .body(req_body)
        .header(Accept::text())
        .header(ContentType(mimetype))
        .send()?;
    try!(map_response_code(res.status(), Some(ApiError::BadBlob)));

    // Read response body containing blob ID
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    BlobId::from_str(body.trim())
}

#[cfg(test)]
mod tests {
    use std::iter::repeat;
    use ::errors::ApiError;
    use super::*;

    #[test]
    fn test_max_length_ok() {
        let text: String = repeat("à").take(3500 / 2).collect();
        let result = send_simple("TESTTEST", &Recipient::new_id("ECHOECHO"), "secret", &text);
        match result {
            Err(ApiError::MessageTooLong) => panic!(),
            _ => (),
        }
    }

    #[test]
    fn test_max_length_too_long() {
        let mut text: String = repeat("à").take(3500 / 2).collect();
        text.push('x');
        let result = send_simple("TESTTEST", &Recipient::new_id("ECHOECHO"), "secret", &text);
        match result {
            Err(ApiError::MessageTooLong) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn test_blob_id_from_str() {
        assert!(BlobId::from_str("0123456789abcdef0123456789abcdef").is_ok());
        assert!(BlobId::from_str("0123456789abcdef0123456789abcdeF").is_ok());
        assert!(BlobId::from_str("0123456789abcdef0123456789abcde").is_err());
        assert!(BlobId::from_str("0123456789abcdef0123456789abcdef\n").is_err());
        assert!(BlobId::from_str("0123456789abcdef0123456789abcdeg").is_err());

        assert_eq!(
            BlobId::from_str("000102030405060708090a0b0c0d0eff").unwrap(),
            BlobId::new([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xff])
        );
    }
}
