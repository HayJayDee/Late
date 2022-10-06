
#[derive(Default)]
pub struct Markup {
    pub keywords: Vec<String>
}

impl Markup {

    pub fn load(file_name: &String) -> Result<Self, std::io::Error>{
        let content = std::fs::read_to_string(file_name)?;
        let mut keywords: Vec<String> = Vec::new();
        for line in content.lines() {
            if line.trim().len() > 0 {
                keywords.push(line.to_string());
            }
        }
        Ok(Self {
            keywords: keywords
        })
    }
}
