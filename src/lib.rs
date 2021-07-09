// Apply tags to filenames in a formatted fashion
// filename[tag tag tag].ext
use std::collections::BTreeSet;
use std::convert::{From, TryFrom};
use std::path::PathBuf;
use std::str::FromStr;

const OPEN_BRACE: u8 = b'[';
const CLOSE_BRACE: u8 = b']';
const DOT: u8 = b'.';
const SPACE: u8 = b' ';
const COMMA: u8 = b',';

type Tag = Vec<u8>;

#[derive(Debug)]
pub struct NameTag {
    prefix: Vec<u8>,
    suffix: Vec<u8>,
    tags: BTreeSet<Tag>,
}

// Interface into tag naming scheme. eg filename[tag1 tag2].ext
impl NameTag {
    pub fn new<T: Into<Vec<u8>>>(name: T) -> Self {
        let data = name.into();
        let mut tags = BTreeSet::new();
        let (prefix, suffix) = match Self::get_tag_bounds(&data) {
            Some((upper, lower)) => {
                Self::parse_tags(&mut tags, &data[upper + 1..lower - 1]);
                // TODO: add tags discovered
                (data[..upper].to_vec(), data[lower..].to_vec())
            }
            _ => {
                // No existing tags. Pick a spot for potential tag insertion.
                // Before the first period, or at the end of the name entirely.
                let split = Self::get_ext_bound(&data);
                (data[..split].to_vec(), data[split..].to_vec())
            }
        };
        Self {
            prefix,
            suffix,
            tags,
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

    pub fn get_tags(&self) -> std::collections::btree_set::Iter<Tag> {
        self.tags.iter()
    }

    /// Remove all tags.
    pub fn clear_tags(&mut self) {
        self.tags.clear();
    }

    fn get_tag_bounds(data: &Vec<u8>) -> Option<(usize, usize)> {
        let mut level = 0;
        let mut last_level = 0;
        let mut upper_bound = 0;
        for (i, ch) in data.iter().enumerate() {
            level += match *ch {
                OPEN_BRACE => 1,
                CLOSE_BRACE => -1,
                _ => 0,
            };
            if last_level < level {
                upper_bound = i;
            }
            if last_level > level {
                return Some((upper_bound, i + 1));
            }
            last_level = level;
        }
        None
    }

    // Find first period, else end of name
    fn get_ext_bound(data: &Vec<u8>) -> usize {
        for (i, ch) in data.iter().enumerate() {
            if *ch == DOT {
                return i;
            }
        }
        data.len()
    }

    fn parse_tags(tags: &mut BTreeSet<Tag>, data: &[u8]) {
        let mut buffer = Vec::new();
        for ch in data.iter() {
            match *ch {
                COMMA | SPACE => {
                    if buffer.len() != 0 {
                        tags.insert(buffer.clone());
                        buffer.clear();
                    }
                }
                ch => buffer.push(ch),
            }
        }
        if 0 < buffer.len() {
            tags.insert(buffer);
        }
    }
}

impl FromStr for NameTag {
    type Err = &'static String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Ok(NameTag::new(name))
    }
}

impl From<NameTag> for Vec<u8> {
    fn from(nametag: NameTag) -> Self {
        let prefix = nametag.prefix.into_iter();
        let suffix = nametag.suffix.into_iter();
        let tag_len = nametag.tags.len();
        if tag_len == 0 {
            prefix.chain(suffix).collect()
        } else {
            let open_brace = Some(OPEN_BRACE).into_iter();
            let close_brace = Some(CLOSE_BRACE).into_iter();
            let tags = nametag.tags.into_iter().enumerate().flat_map(|(i, t)| {
                if i + 1 == tag_len {
                    t.into_iter().chain(None.into_iter())
                } else {
                    t.into_iter().chain(Some(SPACE).into_iter())
                }
            });
            prefix
                .chain(open_brace)
                .chain(tags)
                .chain(close_brace)
                .chain(suffix)
                .collect()
        }
    }
}

impl From<PathBuf> for NameTag {
    fn from(path: PathBuf) -> NameTag {
        NameTag::new(path.as_os_str().as_bytes())
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
        assert_eq!("somefile.txt", &String::try_from(name_tag).unwrap());
    }

    // Functionality
    #[test]
    fn test_get_tags() {
        let name_tag: NameTag = "somefile[tagB tagA].txt".parse().unwrap();
        assert_eq!(
            vec!["tagA", "tagB"],
            name_tag
                .get_tags()
                .map(|t| String::from_utf8_lossy(t))
                .collect::<Vec<_>>()
        );
    }
    #[test]
    fn test_add_tags() {
        let mut name_tag = NameTag::new("somefile.txt");
        name_tag.add_tag("tagB");
        name_tag.add_tag("tagA");
        assert_eq!(
            vec!["tagA", "tagB"],
            name_tag
                .get_tags()
                .map(|t| String::from_utf8_lossy(t))
                .collect::<Vec<_>>()
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
            "somefile[nottag [tagA tagB]].txt",
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
