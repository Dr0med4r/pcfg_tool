use indexmap::IndexSet;

type FoldIndexSet<K> = IndexSet<K, foldhash::fast::RandomState>;

#[derive(Default, Debug, PartialEq, Eq)]
pub struct StringLookup {
    data: FoldIndexSet<String>,
}

impl StringLookup {
    pub fn insert(&mut self, value: String) -> usize {
        self.data.insert_full(value).0
    }

    pub fn get(&self, key: &str) -> std::option::Option<usize> {
        self.data.get_full(key).map(|e| e.0)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get_string(&self, index: usize) -> std::option::Option<&std::string::String> {
        self.data.get_index(index)
    }
}

impl FromIterator<String> for StringLookup {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        StringLookup {
            data: FoldIndexSet::from_iter(iter),
        }
    }
}
