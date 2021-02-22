//! An IME composition string and a candidate list

/// Describes composition character attributes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Attribute {
    Input,
    TargetConverted,
    Converted,
    TargetNotConverted,
    Error,
    FixedConverted,
}

/// A composition character and a composition attribute.
#[derive(Debug)]
pub struct CompositionChar {
    pub ch: char,
    pub attr: Attribute,
}

/// A composition string.
#[derive(Debug)]
pub struct Composition {
    chars: Vec<CompositionChar>,
}

impl Composition {
    pub(crate) fn new(s: String, attrs: Vec<Attribute>) -> Self {
        Self {
            chars: s
                .chars()
                .zip(attrs.into_iter())
                .map(|(ch, attr)| CompositionChar { ch, attr })
                .collect::<Vec<_>>(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    pub fn len(&self) -> usize {
        self.chars.len()
    }

    pub fn iter(&self) -> impl Iterator + '_ {
        self.chars.iter()
    }
}

impl std::iter::IntoIterator for Composition {
    type Item = CompositionChar;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.chars.into_iter()
    }
}

impl std::ops::Index<usize> for Composition {
    type Output = CompositionChar;
    fn index(&self, index: usize) -> &Self::Output {
        &self.chars[index]
    }
}

/// A candidate list.
#[derive(Debug)]
pub struct CandidateList {
    list: Vec<String>,
    selection: usize,
}

impl CandidateList {
    pub(crate) fn new(list: Vec<String>, selection: usize) -> Self {
        Self { list, selection }
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn iter(&self) -> impl Iterator + '_ {
        self.list.iter()
    }

    pub fn selection(&self) -> (usize, &str) {
        (self.selection, &self.list[self.selection])
    }
}

impl std::ops::Index<usize> for CandidateList {
    type Output = str;
    fn index(&self, index: usize) -> &Self::Output {
        &self.list[index]
    }
}
