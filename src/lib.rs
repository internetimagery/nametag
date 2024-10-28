// Apply tags to filenames in a formatted fashion
// filename[tag tag tag].ext
use std::collections::BTreeSet;
use std::convert::{From, TryFrom};
use std::ffi::OsString;
use std::path::PathBuf;
use std::str::FromStr;

type Tag = OsString;

#[derive(Debug)]
pub struct NameTag {
    start: usize,
    stop: usize,
    tags: BTreeSet<Tag>,
    name: OsString,
}

// Interface into tag naming scheme. eg filename[tag1 tag2].ext
impl NameTag {
    pub fn new<T: Into<OsString>>(name: T) -> Self {
        let data = name.into();
        let bytes = data.as_encoded_bytes();
        let mut tags = BTreeSet::new();
        let (start, stop) = match Self::get_tag_bounds(&bytes) {
            Some((upper, lower)) => {
                let blah =
                    unsafe { OsString::from_encoded_bytes_unchecked(bytes[upper..lower].to_vec()) };
                Self::parse_tags(&mut tags, &bytes[upper + 1..lower - 1]);
                (upper, lower)
            }
            _ => {
                // No existing tags. Pick a spot for potential tag insertion.
                // Before the first period, or at the end of the name entirely.
                let split = Self::get_ext_bound(&bytes);
                (split, split)
            }
        };
        Self {
            start,
            stop,
            tags,
            name: data,
        }
    }

    /// Add a new tag. eg tags.add_tag("john")
    pub fn add_tag<T: Into<Tag>>(&mut self, tag: T) {
        self.tags.insert(tag.into());
    }

    /// Remove a tag. eg tags.remove_tag("john")
    pub fn remove_tag<T: Into<Tag>>(&mut self, tag: T) {
        self.tags.remove(&tag.into());
    }

    /// Grab all tags present
    pub fn get_tags(&self) -> std::collections::btree_set::Iter<Tag> {
        self.tags.iter()
    }

    /// Remove all tags.
    pub fn clear_tags(&mut self) {
        self.tags.clear();
        let bytes = self.name.as_encoded_bytes();
        let prefix = bytes[..self.start].iter();
        let suffix = bytes[self.stop..].iter();
        self.name = unsafe {
            OsString::from_encoded_bytes_unchecked(prefix.chain(suffix).copied().collect())
        };
    }

    // Get the in and out of the tag space. eg [ and ]
    fn get_tag_bounds(data: &[u8]) -> Option<(usize, usize)> {
        if let Some(start) = data.iter().position(|x| *x == b'[') {
            if let Some(rstop) = data.iter().rev().position(|x| *x == b']') {
                let stop = data.len() - rstop;
                if start < stop {
                    return Some((start, stop));
                }
            }
        }
        None
    }

    // Find first period, else end of name
    fn get_ext_bound(data: &[u8]) -> usize {
        if let Some(index) = data.iter().position(|x| *x == b'.') {
            return index;
        }
        data.len()
    }

    // Extract tags from name
    fn parse_tags(tags: &mut BTreeSet<Tag>, data: &[u8]) {
        let names = data
            .split(|x| x.is_ascii_whitespace() || *x == b',' || *x == b'[' || *x == b']')
            .filter(|x| x.len() != 0)
            .map(|x| unsafe { OsString::from_encoded_bytes_unchecked(x.to_vec()) });
        tags.extend(names);
    }
}

impl FromStr for NameTag {
    type Err = &'static String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Ok(NameTag::new(OsString::from(name)))
    }
}

impl From<NameTag> for Vec<u8> {
    fn from(nametag: NameTag) -> Self {
        let tag_len = nametag.tags.len();
        if tag_len == 0 {
            nametag.name.as_encoded_bytes().to_vec()
        } else {
            let bytes = nametag.name.as_encoded_bytes();
            let prefix = bytes[..nametag.start].iter();
            let suffix = bytes[nametag.stop..].iter();
            let tags = nametag
                .tags
                .into_iter()
                .collect::<Vec<_>>()
                .join(&OsString::from(" "));

            prefix
                .chain(b"[".into_iter())
                .chain(tags.as_encoded_bytes().iter())
                .chain(b"]".into_iter())
                .chain(suffix)
                .copied()
                .collect()
        }
    }
}

impl From<PathBuf> for NameTag {
    fn from(path: PathBuf) -> NameTag {
        NameTag::new(path.as_os_str())
    }
}

impl TryFrom<NameTag> for String {
    type Error = &'static str;

    fn try_from(value: NameTag) -> Result<Self, Self::Error> {
        if let Ok(result) = String::from_utf8(value.into()) {
            Ok(result)
        } else {
            Err("Failed to convert to string.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    // Basic reading
    #[test]
    fn test_round_trip_no_tags() {
        let name_tag = NameTag::new("somefile.txt");
        assert_eq!("somefile.txt", &String::try_from(name_tag).unwrap());

        let name_tag: NameTag = "somefile.txt".parse().unwrap();
        assert_eq!("somefile.txt", &String::try_from(name_tag).unwrap());
    }
    #[test]
    fn test_round_trip_maintain_tags() {
        let name_tag: NameTag = "somefile[tagB tagA].txt".parse().unwrap();
        assert_eq!(
            "somefile[tagA tagB].txt",
            &String::try_from(name_tag).unwrap()
        );
    }
    #[test]
    fn test_round_trip_empty_tags() {
        let name_tag: NameTag = "somefile[].txt".parse().unwrap();
        assert_eq!("somefile[].txt", &String::try_from(name_tag).unwrap());
    }

    // Functionality
    #[test]
    fn test_get_tags() {
        let name_tag: NameTag = "somefile[tagB tagA].txt".parse().unwrap();
        assert_eq!(
            vec!["tagA", "tagB"],
            name_tag.get_tags().collect::<Vec<_>>()
        );
    }
    #[test]
    fn test_add_tags() {
        let mut name_tag = NameTag::new("somefile.txt");
        name_tag.add_tag("tagB");
        name_tag.add_tag("tagA");
        assert_eq!(
            vec!["tagA", "tagB"],
            name_tag.get_tags().collect::<Vec<_>>()
        );
    }
    #[test]
    fn test_round_trip_add_tag() {
        let mut name_tag = NameTag::new("somefile[tagB].txt");
        name_tag.add_tag("tagA");
        assert_eq!(
            "somefile[tagA tagB].txt",
            &String::try_from(name_tag).unwrap()
        );
    }
    #[test]
    fn test_round_trip_remove_tag() {
        let mut name_tag = NameTag::new("somefile[tagB tagA].txt");
        name_tag.remove_tag("tagA");
        assert_eq!("somefile[tagB].txt", &String::try_from(name_tag).unwrap());
    }
    #[test]
    fn test_round_trip_remove_absent_tag() {
        let mut name_tag = NameTag::new("somefile[tagB tagA].txt");
        name_tag.remove_tag("tagC");
        assert_eq!(
            "somefile[tagA tagB].txt",
            &String::try_from(name_tag).unwrap()
        );
    }
    #[test]
    fn test_round_trip_clear_tags() {
        let mut name_tag = NameTag::new("somefile[tagB tagA].txt");
        name_tag.clear_tags();
        assert_eq!("somefile.txt", &String::try_from(name_tag).unwrap());
    }

    // Edgy Cases
    #[test]
    fn test_round_trip_nested_braces() {
        let name_tag: NameTag = "somefile[nottag [tagB tagA]].txt".parse().unwrap();
        assert_eq!(
            "somefile[nottag tagA tagB].txt",
            &String::try_from(name_tag).unwrap()
        );
    }
    #[test]
    fn test_round_trip_unmatched_braces() {
        let name_tag: NameTag = "somefile[tagB tagA.txt".parse().unwrap();
        assert_eq!(
            "somefile[tagB tagA.txt",
            &String::try_from(name_tag).unwrap()
        );
    }
    #[test]
    fn test_round_trip_lota_spaces() {
        let name_tag: NameTag = "somefile[   tagB    tagA  ].txt".parse().unwrap();
        assert_eq!(
            "somefile[tagA tagB].txt",
            &String::try_from(name_tag).unwrap()
        );
    }
    #[test]
    fn test_round_trip_tags_in_front() {
        let name_tag: NameTag = "[tagB tagA]somefile.txt".parse().unwrap();
        assert_eq!(
            "[tagA tagB]somefile.txt",
            &String::try_from(name_tag).unwrap()
        );
    }
}
