use std::{collections::HashMap, ops::Index};

#[derive(Debug)]
pub struct Path {
    raw_value: String,
    segments: Option<Vec<String>>,
    labels: Option<HashMap<String, usize>>,
    has_wild_card: bool,
}

impl TryFrom<String> for Path {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let raw_value = value.clone();
        if !raw_value.contains("{") {
            return Ok(Self {
                raw_value,
                has_wild_card: false,
                labels: None,
                segments: None,
            });
        }

        let mut segments: Vec<String> = Vec::new();
        let mut labels: HashMap<String, usize> = HashMap::new();

        let mut segment_count = 0;
        let mut segment = "".to_string();
        let mut label = "".to_string();
        let mut in_label = false;
        for c in value.chars() {
            if c.is_whitespace() {
                return Err("Path cannot contain whitespaces".to_string());
            }
            if c == '{' {
                if in_label {
                    return Err(format!("Mal formed path: {}", value));
                }
                in_label = true;
                segments.push(segment);
                segment = "".to_string();
                continue;
            }
            if c == '}' {
                if !in_label {
                    return Err(format!("Mal formed path: {}", value));
                }
                if labels.contains_key(&label) {
                    return Err(format!("duplicated label: {}", value));
                }
                labels.insert(label.clone(), segment_count);
                label = "".to_string();
                segment_count += 1;
                in_label = false;
                continue;
            }

            if in_label {
                label.push(c);
                continue;
            }

            segment.push(c);
        }

        if in_label {
            return Err(format!("Mal formed path: {}", value));
        }

        if segment != "" {
            segments.push(segment);
        }

        Ok(Self {
            raw_value,
            segments: Some(segments),
            labels: Some(labels),
            has_wild_card: true,
        })
    }
}

impl TryFrom<&str> for Path {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        if self.raw_value == other.raw_value {
            return true;
        }

        let segments: &Vec<String>;
        let mut raw_value: &str;
        match (&self.segments, &other.segments) {
            (None, None) => {
                //If none have wild_cards they can't be equal because they are simple paths and we already check the raw values
                return false;
            }
            (Some(s1), Some(s2)) => {
                //If they have the same segments they are equal, this is important because 2 paths with the same segments and different labels should error when trying to add
                //because they are in conflict with each other
                return s1 == s2;
            }
            (Some(s1), None) => {
                segments = s1;
                raw_value = &other.raw_value;
            }
            (None, Some(s2)) => {
                segments = s2;
                raw_value = &self.raw_value;
            }
        };

        for segment in segments {
            let index = match raw_value.find(segment) {
                Some(i) => i,
                None => return false,
            };
            let rest = &raw_value[..index];
            if rest != "" {
                //this is a partial match
                return false;
            }
            raw_value = &raw_value[index + segment.len()..];
            let next_segment_index = raw_value.find("/").unwrap_or(0);
            raw_value = &raw_value[next_segment_index..];
        }

        return true;
    }
}

impl Eq for Path {}

#[cfg(test)]
mod test {
    use crate::server::path::Path;

    #[test]
    fn test_equal_simple_paths() {
        let base = "/users";
        let p1 = Path::try_from(base).unwrap();
        let p2 = Path::try_from(base).unwrap();
        assert_eq!(p1, p2)
    }

    #[test]
    fn create_path_with_one_label() {
        let base = "/users/{id}";
        let p1 = Path::try_from(base).unwrap();
        assert_eq!(p1.raw_value, base.to_string());
        assert!(p1.has_wild_card);
        assert!(p1.segments.is_some());
        assert!(p1.labels.is_some());

        let segments = p1.segments.unwrap();
        let labels = p1.labels.unwrap();

        assert_eq!(segments.len(), 1);
        assert_eq!(labels.len(), 1);
        assert_eq!(segments[0], "/users/");
        assert_eq!(*labels.get("id").unwrap(), 0);
    }

    #[test]
    fn create_path_with_one_label_and_trailing_segment() {
        let base = "/users/{id}/some";
        let p1 = Path::try_from(base).unwrap();
        assert_eq!(p1.raw_value, base.to_string());
        assert!(p1.has_wild_card);
        assert!(p1.segments.is_some());
        assert!(p1.labels.is_some());

        let segments = p1.segments.unwrap();
        let labels = p1.labels.unwrap();

        assert_eq!(segments.len(), 2);
        assert_eq!(labels.len(), 1);
        assert_eq!(segments[0], "/users/");
        assert_eq!(segments[1], "/some");
        assert_eq!(*labels.get("id").unwrap(), 0);
    }

    #[test]
    fn create_path_with_two_labels() {
        let base = "/users/{id}/{name}";
        let p1 = Path::try_from(base).unwrap();
        assert_eq!(p1.raw_value, base.to_string());
        assert!(p1.has_wild_card);
        assert!(p1.segments.is_some());
        assert!(p1.labels.is_some());

        let segments = p1.segments.unwrap();
        let labels = p1.labels.unwrap();

        assert_eq!(segments.len(), 2);
        assert_eq!(labels.len(), 2);
        assert_eq!(segments[0], "/users/");
        assert_eq!(segments[1], "/");
        assert_eq!(*labels.get("id").unwrap(), 0);
        assert_eq!(*labels.get("name").unwrap(), 1);
    }

    #[test]
    fn create_path_with_two_labels_with_trailing_segment() {
        let base = "/users/{id}/{name}/some";
        let p1 = Path::try_from(base).unwrap();
        assert_eq!(p1.raw_value, base.to_string());
        assert!(p1.has_wild_card);
        assert!(p1.segments.is_some());
        assert!(p1.labels.is_some());

        let segments = p1.segments.unwrap();
        let labels = p1.labels.unwrap();

        assert_eq!(segments.len(), 3);
        assert_eq!(labels.len(), 2);
        assert_eq!(segments[0], "/users/");
        assert_eq!(segments[1], "/");
        assert_eq!(segments[2], "/some");
        assert_eq!(*labels.get("id").unwrap(), 0);
        assert_eq!(*labels.get("name").unwrap(), 1);
    }

    #[test]
    fn test_equal_path_with_wild_cards() {
        let p1 = "/users/{id}";
        let p1 = Path::try_from(p1).unwrap();
        let p2 = "/users/1";
        let p2 = Path::try_from(p2).unwrap();
        assert_eq!(p1, p2)
    }

    #[test]
    fn test_equal_path_with_wild_cards_with_trailing_segment() {
        let p1 = "/users/{id}/some";
        let p1 = Path::try_from(p1).unwrap();
        let p2 = "/users/1/some";
        let p2 = Path::try_from(p2).unwrap();
        assert_eq!(p1, p2)
    }

    #[test]
    fn test_equal_path_with_wild_cards_with_trailing_segment_but_not_equal_1() {
        let p1 = "/users/{id}/some";
        let p1 = Path::try_from(p1).unwrap();
        let p2 = "/users/1";
        let p2 = Path::try_from(p2).unwrap();
        assert_ne!(p1, p2);
    }

    #[test]
    fn test_equal_path_with_wild_cards_with_trailing_segment_but_not_equal_2() {
        let p1 = "/users/{id}";
        let p1 = Path::try_from(p1).unwrap();
        let p2 = "/users/1/some";
        let p2 = Path::try_from(p2).unwrap();
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_equal_path_2_wild_cards_and_trailing_segment() {
        let p1 = "/users/{id}/info/{name}/some";
        let p1 = Path::try_from(p1).unwrap();
        let p2 = "/users/1/info/mario/some";
        let p2 = Path::try_from(p2).unwrap();
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_equal_path_2_wild_cards_and_trailing_segment_2() {
        let p1 = "/users/{id}/info/{name}/some";
        let p1 = Path::try_from(p1).unwrap();
        let p2 = "/users/1/mario/info/some";
        let p2 = Path::try_from(p2).unwrap();
        assert_ne!(p1, p2);
    }

    #[test]
    fn test_equal_same_segments_different_labels() {
        let p1 = "/users/{id}/info/{name}/some";
        let p1 = Path::try_from(p1).unwrap();
        let p2 = "/users/{name}/info/{id}/some";
        let p2 = Path::try_from(p2).unwrap();
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_mal_formed_path() {
        let p1 = "/users/{id}/info/{name";
        Path::try_from(p1).unwrap_err();
    }

    #[test]
    fn test_mal_formed_path_2() {
        let p1 = "/users/{id}}";
        Path::try_from(p1).unwrap_err();
    }

    #[test]
    fn test_mal_formed_path_3() {
        let p1 = "/users/{{id}";
        Path::try_from(p1).unwrap_err();
    }

    #[test]
    fn test_mal_formed_path_4() {
        let p1 = "/users/{id}/{id}";
        Path::try_from(p1).unwrap_err();
    }
}
