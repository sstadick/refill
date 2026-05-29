use facet::ReflectError;

#[derive(Debug, Clone)]
pub struct TemplateError {
    pub kind: TemplateErrorKind,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum TemplateErrorKind {
    InvalidContextError { reflect_error: ReflectError },
    MissingFieldError { expected: String },
}

impl core::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl core::fmt::Display for TemplateErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateErrorKind::InvalidContextError { reflect_error } => {
                write!(f, "Invalid Ctx type: {}", reflect_error)?
            }
            TemplateErrorKind::MissingFieldError { expected } => {
                write!(f, "Expected a field on Ctx for {}, none found.", expected)?
            }
        }
        Ok(())
    }
}
