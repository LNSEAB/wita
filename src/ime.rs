#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Attribute {
    Input,
    TargetConverted,
    Converted,
    TargetNotConverted,
    Error,
    FixedConverted,
}

#[derive(Debug)]
pub struct CompositionChar {
    pub ch: char,
    pub attr: Attribute,
}

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

    pub fn into_iter(self) -> impl IntoIterator {
        self.chars.into_iter()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator + 'a {
        self.chars.iter()
    }

    pub fn to_string(&self) -> String {
        self.chars.iter().map(|elem| elem.ch).collect::<String>()
    }
}

impl std::ops::Index<usize> for Composition {
    type Output = CompositionChar;
    fn index(&self, index: usize) -> &Self::Output {
        &self.chars[index]
    }
}

#[derive(Debug)]
pub struct CandidateList {
    list: Vec<String>,
    selection: usize,
}

impl CandidateList {
    pub(crate) fn new(list: Vec<String>, selection: usize) -> Self {
        Self { list, selection }
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator + 'a {
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
