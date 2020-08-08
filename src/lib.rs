// Apply tags to filenames in a formatted fashion
// filename[tag tag tag].ext
// [tag tag tag].ext

const OPEN_BRACE: u8 = b'[';
const CLOSE_BRACE: u8 = b'[';

#[derive(Debug)]
pub struct NameTag {
    name: Vec<u8>,
    tags: Vec<Vec<u8>>,
}

impl NameTag {
    pub fn new<T: Into<Vec<u8>>>(name: T)  -> Self {
        let data = name.into();
        let tags = Vec::new();
        if let Some(bounds) = Self::get_tag_bounds(&data) {
        }
        Self{name: data, tags: tags}
    }

    pub fn add_tag<T: Into<Vec<u8>>>(&mut self, tag: T) {
        println!("Adding {:?}", tag.into());

    }

    pub fn remove_tag<T: Into<Vec<u8>>>(&mut self, tag: T) {

    }

    pub fn get_tags(&self) -> std::slice::Iter<Vec<u8>> {
        self.tags.iter()
    }

    fn get_tag_bounds(data: &Vec<u8>) -> Option<usize> {
        for (i, ch) in data.iter().enumerate() {
            
        }
        None
    }
}

impl std::convert::TryFrom<NameTag> for String {
    type Error = &'static str;

    fn try_from(value: NameTag) -> Result<Self, Self::Error> {
        if let Ok(result) = String::from_utf8(value.name) {
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

    #[test]
    fn test_round_trip_no_change() {
        let name_tag = NameTag::new("somefile.txt");
        assert_eq!("somefile.txt", &String::try_from(name_tag).unwrap());
    }
    #[test]
    fn test_round_trip_maintain_tags() {
        let name_tag = NameTag::new("somefile[tagb taga].txt");
        assert_eq!(vec!["tagb", "taga"], name_tag.get_tags().map(|t| String::from_utf8_lossy(t)).collect::<Vec<_>>());
        assert_eq!("somefile[tagb taga].txt", &String::try_from(name_tag).unwrap());
    }
    
    #[test]
    fn test_round_trip_add_tag() {
        let mut name_tag = NameTag::new("somefile.txt");
        name_tag.add_tag("hello");
        assert_eq!("somefile[hello].txt", &String::try_from(name_tag).unwrap());
    }

}