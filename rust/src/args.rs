
pub struct Args {
    args: Vec<String>,
}


impl Args {
    pub fn new() -> Self {
        Self {
            args: Vec::new(),
        }
    }
    pub fn add(&mut self, arg: impl AsRef<str>) -> &mut Self {
        self.args.push(arg.as_ref().to_owned());
        self
    }
    pub fn extend<T>(&mut self, args: T) -> &mut Self
        where T: IntoIterator,
              <T as IntoIterator>::Item: AsRef<str>
    {
        args.into_iter().for_each(|s| { self.add(s); });
        self
    }
    pub fn get(&self)-> &[String] {
        &self.args
    }
    pub fn into_vec(self) -> Vec<String> {
        self.args
    }
}

impl<T> From<T> for Args
where T: IntoIterator,
      <T as IntoIterator>::Item: AsRef<str>
{
    fn from(value: T) -> Self {
        Self {
            args: value.into_iter()
                .map(|s| s.as_ref().to_owned())
                .collect()
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}

