use crate::providers::Provider;
use crate::providers::Item;

#[derive(Debug)]
pub struct Wireplumber {
    name: &'static str,
}

impl Wireplumber {
    pub fn new() -> Self {
        Self {
            name: "wireplumber",
        }
    }
}

impl Provider for Wireplumber {
    fn get_name(&self) -> &str {
        self.name
    }

    fn get_item_layout(&self) -> String {
        include_str!("../../resources/themes/default/item_progress.xml").to_string()
    }

    fn progress_transformer(&self, item: &Item) -> f64 {
        //TODO(amatej): parse only the string ending
        let num_str: String = item.subtext.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
        let val = num_str.parse::<f64>().unwrap();
        (val / 100.0).clamp(0.0, 1.0)
    }
}
