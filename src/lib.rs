//! Provides a lightweight wrapper for the DeepL Pro REST API.
//!
//! *If you are looking for the `deepl` commandline utitlity, please refer
//! to [its documentation](../deepl/index.html) instead.*
//!
//! # Requirements
//!
//! You need to have a valid [DeepL Pro Developer](https://www.deepl.com/pro#developer) account
//! with an associated API key. This key must be made available to the application, e. g. via
//! environment variable:
//!
//! ```bash
//! export DEEPL_API_KEY=YOUR_KEY
//! ```
//!
//! # Example
//!
//! ```rust
//! use deepl_api::*;
//!
//! // Create a DeepL instance for our account.
//! let deepl = DeepL::new(std::env::var("DEEPL_API_KEY").unwrap());
//!
//! // Translate Text
//! let texts = TranslatableTextList {
//!     source_language: Some("DE".to_string()),
//!     target_language: "EN-US".to_string(),
//!     texts: vec!("ja".to_string()),
//! };
//! let translated = deepl.translate(None, texts).unwrap();
//! assert_eq!(translated[0].text, "yes");
//!
//! // Fetch Usage Information
//! let usage_information = deepl.usage_information().unwrap();
//! assert!(usage_information.character_limit > 0);
//! ```
//!
//! # See Also
//!
//! The main API functions are documented in the [DeepL] struct.

use error_chain::*;
use reqwest;
use serde::Deserialize;

/// Information about API usage & limits for this account.
#[derive(Debug, Deserialize)]
pub struct UsageInformation {
    /// How many characters can be translated per billing period, based on the account settings.
    pub character_limit: u64,
    /// How many characters were already translated in the current billing period.
    pub character_count: u64,
}

/// Information about available languages.
pub type LanguageList = Vec<LanguageInformation>;

/// Information about a single language.
#[derive(Debug, Deserialize)]
pub struct LanguageInformation {
    /// Custom language identifier used by DeepL, e. g. "EN-US". Use this
    /// when specifying source or target language.
    pub language: String,
    /// English name of the language, e. g. `English (America)`.
    pub name: String,
}

/// Translation option that controls the splitting of sentences before the translation.
pub enum SplitSentences {
    /// Don't split sentences.
    None,
    /// Split on punctiation only.
    Punctuation,
    /// Split on punctuation and newlines.
    PunctuationAndNewlines,
}

/// Translation option that controls the desired translation formality.
pub enum Formality {
    /// Default formality.
    Default,
    /// Translate less formally.
    More,
    /// Translate more formally.
    Less,
}

/// Custom [flags for the translation request](https://www.deepl.com/docs-api/translating-text/request/).
pub struct TranslationOptions {
    /// Sets whether the translation engine should first split the input into sentences. This is enabled by default.
    pub split_sentences: Option<SplitSentences>,
    /// Sets whether the translation engine should respect the original formatting, even if it would usually correct some aspects.
    pub preserve_formatting: Option<bool>,
    /// Sets whether the translated text should lean towards formal or informal language.
    pub formality: Option<Formality>,
}

/// Holds a list of strings to be translated.
#[derive(Debug, Deserialize)]
pub struct TranslatableTextList {
    /// Source language, if known. Will be auto-detected by the DeepL API
    /// if not provided.
    pub source_language: Option<String>,
    /// Target language (required).
    pub target_language: String,
    /// List of texts that are supposed to be translated.
    pub texts: Vec<String>,
}

/// Holds one unit of translated text.
#[derive(Debug, Deserialize, PartialEq)]
pub struct TranslatedText {
    /// Source language. Holds the value provided, or otherwise the value that DeepL auto-detected.
    pub detected_source_language: String,
    /// Translated text.
    pub text: String,
}

// Only needed for JSON deserialization.
#[derive(Debug, Deserialize)]
struct TranslatedTextList {
    translations: Vec<TranslatedText>,
}

// Only needed for JSON deserialization.
#[derive(Debug, Deserialize)]
struct ServerErrorMessage {
    message: String,
}

/// The main API entry point representing a DeepL developer account with an associated API key.
///
/// # Example
///
/// See [Example](crate#example).
///
/// # Error Handling
///
/// None of the functions will panic. Instead, the API methods usually return a [Result<T>] which may
/// contain an [Error] of one of the defined [ErrorKinds](ErrorKind) with more information about what went wrong.
///
/// If you get an [AuthorizationError](ErrorKind::AuthorizationError), then something was wrong with your API key, for example.
pub struct DeepL {
    api_key: String,
}

/// Implements the actual REST API. See also the [online documentation](https://www.deepl.com/docs-api/).
impl DeepL {
    /// Use this to create a new DeepL API client instance where multiple function calls can be performed.
    /// A valid `api_key` is required.
    ///
    /// Should you ever need to use more than one DeepL account in our program, then you can create one
    /// instance for each account / API key.
    pub fn new(api_key: String) -> DeepL {
        DeepL { api_key }
    }

    /// Private method that performs the HTTP calls.
    fn http_request(
        &self,
        url: &str,
        query: &Vec<(&str, std::string::String)>,
    ) -> Result<reqwest::blocking::Response> {

        let url = format!("https://api.deepl.com/v2{}", url);
        let mut payload = query.clone();
        payload.push(("auth_key", self.api_key.clone()));

        let client = reqwest::blocking::Client::new();

        let res = match client.post(&url).query(&payload).send() {
            Ok(response) if response.status().is_success() => response,
            Ok(response) if response.status() == reqwest::StatusCode::UNAUTHORIZED => {
                bail!(ErrorKind::AuthorizationError)
            }
            Ok(response) if response.status() == reqwest::StatusCode::FORBIDDEN => {
                bail!(ErrorKind::AuthorizationError)
            }
            // DeepL sends back error messages in the response body.
            //   Try to fetch them to construct more helpful exceptions.
            Ok(response) => {
                let status = response.status();
                match response.json::<ServerErrorMessage>() {
                    Ok(server_error) => bail!(ErrorKind::ServerError(server_error.message)),
                    _ => bail!(ErrorKind::ServerError(status.to_string())),
                }
            }
            Err(e) => {
                bail!(e)
            }
        };
        Ok(res)
    }

    /// Retrieve information about API usage & limits.
    /// This can also be used to verify an API key without consuming translation contingent.
    ///
    /// See also the [vendor documentation](https://www.deepl.com/docs-api/other-functions/monitoring-usage/).
    pub fn usage_information(&self) -> Result<UsageInformation> {
        let res = self.http_request("/usage", &vec![])?;

        match res.json::<UsageInformation>() {
            Ok(content) => return Ok(content),
            _ => {
                bail!(ErrorKind::DeserializationError);
            }
        };
    }

    /// Retrieve all currently available source languages.
    ///
    /// See also the [vendor documentation](https://www.deepl.com/docs-api/other-functions/listing-supported-languages/).
    pub fn source_languages(&self) -> Result<LanguageList> {
        return self.languages("source");
    }

    /// Retrieve all currently available target languages.
    ///
    /// See also the [vendor documentation](https://www.deepl.com/docs-api/other-functions/listing-supported-languages/).
    pub fn target_languages(&self) -> Result<LanguageList> {
        return self.languages("target");
    }

    /// Private method to make the API calls for the language lists.
    fn languages(&self, language_type: &str) -> Result<LanguageList> {
        let res = self.http_request("/languages", &vec![("type", language_type.to_string())])?;

        match res.json::<LanguageList>() {
            Ok(content) => return Ok(content),
            _ => bail!(ErrorKind::DeserializationError),
        }
    }

    /// Translate one or more [text chunks](TranslatableTextList) at once. You can pass in optional
    /// [translation flags](TranslationOptions) if you need non-default behaviour.
    ///
    /// Please see the parameter documentation and the
    /// [vendor documentation](https://www.deepl.com/docs-api/translating-text/) for details.
    pub fn translate(
        &self,
        options: Option<TranslationOptions>,
        text_list: TranslatableTextList,
    ) -> Result<Vec<TranslatedText>> {
        let mut query = vec![
            ("target_lang", text_list.target_language),
        ];
        if let Some(source_language_content) = text_list.source_language {
            query.push(("source_lang", source_language_content));
        }
        for text in text_list.texts {
            query.push(("text", text));
        }
        if let Some(opt) = options {
            if let Some(split_sentences) = opt.split_sentences {
                query.push((
                    "split_sentences",
                    match split_sentences {
                        SplitSentences::None => "0".to_string(),
                        SplitSentences::PunctuationAndNewlines => "1".to_string(),
                        SplitSentences::Punctuation => "nonewlines".to_string(),
                    },
                ));
            }
            if let Some(preserve_formatting) = opt.preserve_formatting {
                query.push((
                    "preserve_formatting",
                    match preserve_formatting {
                        false => "0".to_string(),
                        true => "1".to_string(),
                    },
                ));
            }
            if let Some(formality) = opt.formality {
                query.push((
                    "formality",
                    match formality {
                        Formality::Default => "default".to_string(),
                        Formality::More => "more".to_string(),
                        Formality::Less => "less".to_string(),
                    },
                ));
            }
        }

        let res = self.http_request("/translate", &query)?;

        match res.json::<TranslatedTextList>() {
            Ok(content) => Ok(content.translations),
            _ => bail!(ErrorKind::DeserializationError),
        }
    }
}

mod errors {
    use error_chain::*;
    error_chain! {}
}

pub use errors::*;

error_chain! {
    foreign_links {
        IO(std::io::Error);
        Transport(reqwest::Error);
    }
    errors {
        /// Indicates that the provided API key was refused by the DeepL server.
        AuthorizationError {
            description("Authorization failed, is your API key correct?")
            display("Authorization failed, is your API key correct?")
        }
        /// An error occurred on the server side when processing a request. If possible, details
        /// will be provided in the error message.
        ServerError(message: String) {
            description("An error occurred while communicating with the DeepL server.")
            display("An error occurred while communicating with the DeepL server: '{}'.", message)
        }
        /// An error occurred on the client side when deserializing the response data.
        DeserializationError {
            description("An error occurred while deserializing the response data.")
            display("An error occurred while deserializing the response data.")
        }
    }

    skip_msg_variant
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_information() {
        let key = std::env::var("DEEPL_API_KEY").unwrap();
        let usage_information = DeepL::new(key).usage_information().unwrap();
        assert!(usage_information.character_limit > 0);
    }

    #[test]
    fn source_languages() {
        let key = std::env::var("DEEPL_API_KEY").unwrap();
        let source_languages = DeepL::new(key).source_languages().unwrap();
        assert_eq!(source_languages.last().unwrap().name, "Chinese");
    }

    #[test]
    fn target_languages() {
        let key = std::env::var("DEEPL_API_KEY").unwrap();
        let target_languages = DeepL::new(key).target_languages().unwrap();
        assert_eq!(target_languages.last().unwrap().name, "Chinese");
    }

    #[test]
    fn translate() {
        let key = std::env::var("DEEPL_API_KEY").unwrap();
        let deepl = DeepL::new(key);
        let tests = vec![
            (
                None,
                TranslatableTextList {
                    source_language: Some("DE".to_string()),
                    target_language: "EN-US".to_string(),
                    texts: vec!["ja".to_string()],
                },
                vec![TranslatedText {
                    detected_source_language: "DE".to_string(),
                    text: "yes".to_string(),
                }],
            ),
            (
                Some(TranslationOptions {
                    split_sentences: None,
                    preserve_formatting: Some(true),
                    formality: None,
                }),
                TranslatableTextList {
                    source_language: Some("DE".to_string()),
                    target_language: "EN-US".to_string(),
                    texts: vec!["ja\n nein".to_string()],
                },
                vec![TranslatedText {
                    detected_source_language: "DE".to_string(),
                    text: "yes\n no".to_string(),
                }],
            ),
            (
                Some(TranslationOptions {
                    split_sentences: Some(SplitSentences::None),
                    preserve_formatting: None,
                    formality: None,
                }),
                TranslatableTextList {
                    source_language: Some("DE".to_string()),
                    target_language: "EN-US".to_string(),
                    texts: vec!["Ja. Nein.".to_string()],
                },
                vec![TranslatedText {
                    detected_source_language: "DE".to_string(),
                    text: "Yes. No.".to_string(),
                }],
            ),
            (
                Some(TranslationOptions {
                    split_sentences: None,
                    preserve_formatting: None,
                    formality: Some(Formality::More),
                }),
                TranslatableTextList {
                    source_language: Some("EN".to_string()),
                    target_language: "DE".to_string(),
                    texts: vec!["Please go home.".to_string()],
                },
                vec![TranslatedText {
                    detected_source_language: "EN".to_string(),
                    text: "Bitte gehen Sie nach Hause.".to_string(),
                }],
            ),
            (
                Some(TranslationOptions {
                    split_sentences: None,
                    preserve_formatting: None,
                    formality: Some(Formality::Less),
                }),
                TranslatableTextList {
                    source_language: Some("EN".to_string()),
                    target_language: "DE".to_string(),
                    texts: vec!["Please go home.".to_string()],
                },
                vec![TranslatedText {
                    detected_source_language: "EN".to_string(),
                    text: "Bitte geh nach Hause.".to_string(),
                }],
            ),
        ];
        for test in tests {
            assert_eq!(deepl.translate(test.0, test.1).unwrap(), test.2);
        }
    }

    #[test]
    #[should_panic(expected = "Error(ServerError(\"Parameter \\'text\\' not specified.")]
    fn translate_empty() {
        let key = std::env::var("DEEPL_API_KEY").unwrap();
        let texts = TranslatableTextList {
            source_language: Some("DE".to_string()),
            target_language: "EN-US".to_string(),
            texts: vec![],
        };
        DeepL::new(key).translate(None, texts).unwrap();
    }

    #[test]
    #[should_panic(expected = "Error(ServerError(\"Value for \\'target_lang\\' not supported.")]
    fn translate_wrong_language() {
        let key = std::env::var("DEEPL_API_KEY").unwrap();
        let texts = TranslatableTextList {
            source_language: None,
            target_language: "NONEXISTING".to_string(),
            texts: vec!["ja".to_string()],
        };
        DeepL::new(key).translate(None, texts).unwrap();
    }

    #[test]
    #[should_panic(expected = "Error(AuthorizationError")]
    fn translate_unauthorized() {
        let key = "wrong_key".to_string();
        let texts = TranslatableTextList {
            source_language: Some("DE".to_string()),
            target_language: "EN-US".to_string(),
            texts: vec!["ja".to_string()],
        };
        DeepL::new(key).translate(None, texts).unwrap();
    }
}
