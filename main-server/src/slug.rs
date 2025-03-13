use std::fmt::Display;

pub struct Slug<'a>(pub &'a str);

impl Display for Slug<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&slug::slugify(self.0))?;
        Ok(())
    }
}
