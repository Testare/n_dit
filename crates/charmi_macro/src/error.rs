use proc_macro2::Span;

#[derive(Default)]
pub struct CharmiError {
    unexpected_keys: Vec<String>,
    simple_errors: Vec<String>,
}

impl CharmiError {
    pub fn none_if_empty(self) -> Option<Self> {
        if self.unexpected_keys.is_empty() && self.simple_errors.is_empty() {
            None
        } else {
            Some(self)
        }
    }

    pub fn unexpected_key(key: &str) -> Option<Self> {
        Some(Self {
            unexpected_keys: vec![key.to_string()],
            simple_errors: Vec::new(),
        })
    }

    pub fn wrong_type(key: &str, expected: &str, actual: &str) -> Option<Self> {
        let message = format!("{key} was type {actual}, expected {expected}");
        Some(Self {
            simple_errors: vec![message],
            unexpected_keys: Vec::new(),
        })
    }

    pub fn char_length_error(key: &str, len: usize) -> Option<Self> {
        let plural = if len == 1 { "" } else { "s" };
        let message = format!("{key} should be {len} single-width character{plural}");
        Some(Self {
            simple_errors: vec![message],
            unexpected_keys: Vec::new(),
        })
    }

    pub fn unrecognized_color(color: &str) -> Option<Self> {
        Some(Self {
            simple_errors: vec![format!("Unrecognized color: {}", color)],
            unexpected_keys: Vec::new(),
        })
    }

    pub fn unrecognized_attr(attr: &str) -> Option<Self> {
        Some(Self {
            simple_errors: vec![format!(
                "Unrecognized attr: {}, should be underline or bold",
                attr
            )],
            unexpected_keys: Vec::new(),
        })
    }

    pub fn to_syn_error(&self, span: Span) -> syn::Result<()> {
        let mut errors: Vec<String> = Vec::new();
        if !self.unexpected_keys.is_empty() {
            errors.push(format!("Unexpected_keys: {:?}", self.unexpected_keys));
        }
        errors.extend(self.simple_errors.clone());
        if !errors.is_empty() {
            // TODO syn::Error::combine instead?
            Err(syn::Error::new(
                span,
                format!("TOML doesn't match charmi spec: {errors:?}"),
            ))
        } else {
            Ok(())
        }
    }
}

impl std::ops::Add<CharmiError> for CharmiError {
    type Output = Self;
    fn add(mut self, rhs: CharmiError) -> Self::Output {
        let Self {
            unexpected_keys,
            simple_errors,
        } = rhs;
        self.unexpected_keys.extend(unexpected_keys);
        self.simple_errors.extend(simple_errors);
        self
    }
}

impl std::iter::Sum for CharmiError {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result: Self = Default::default();
        for Self {
            unexpected_keys,
            simple_errors,
        } in iter
        {
            result.unexpected_keys.extend(unexpected_keys);
            result.simple_errors.extend(simple_errors);
        }
        result
    }
}
