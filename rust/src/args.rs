//! Convenient data type to represent command-line arguments.

/**
 * A struct representing command-line argument list.
 */
#[derive(Default)]
pub struct Args {
    args: Vec<String>,
}


impl Args {
    /**
     * Creates a new, empty list.
     */
    pub fn new() -> Self {
        Default::default()
    }

    /**
     * Adds a single argument to the list.
     */
    pub fn add(&mut self, arg: impl AsRef<str>) -> &mut Self {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    /**
     * Adds arguments to the list from the given iterator.
     */
    pub fn extend<T>(&mut self, args: T) -> &mut Self
        where T: IntoIterator,
              <T as IntoIterator>::Item: AsRef<str>
    {
        args.into_iter().for_each(|s| { self.add(s); });
        self
    }

    /**
     * Gets a reference to the argument list.
     */
    pub fn get(&self)-> &[String] {
        &self.args
    }

    /**
     * Consumes the argument list and returns it as a vector.
     */
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


