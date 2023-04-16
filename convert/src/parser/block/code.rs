pub(crate) type LanguageIdentifier = String;

#[derive(Debug)]
pub(crate) struct CodeBlock {
    language: Option<LanguageIdentifier>,
    src: String,
}

impl CodeBlock {
    pub fn new(language: Option<LanguageIdentifier>, src: String) -> Self {
        Self { language, src }
    }

    pub fn language(&self) -> &Option<LanguageIdentifier> {
        &self.language
    }

    pub fn src(&self) -> &str {
        &self.src
    }
}
